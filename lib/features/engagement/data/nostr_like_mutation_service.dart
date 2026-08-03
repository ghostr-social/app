import 'package:ghostr/core/async/keyed_serial_task_queue.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_client.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/engagement/data/accepted_nostr_reaction_journal.dart';
import 'package:ghostr/features/engagement/data/nostr_engagement_reader.dart';
import 'package:ghostr/features/engagement/data/nostr_reaction_target.dart';
import 'package:ghostr/features/engagement/domain/nostr_engagement_port.dart';
import 'package:ghostr/features/engagement/domain/video_engagement.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';

class NostrLikeMutationService {
  const NostrLikeMutationService(
    this._client,
    this._reader,
    this._journal,
    this._queue,
  );

  final NostrEventClient _client;
  final NostrEngagementReader _reader;
  final AcceptedNostrReactionJournal _journal;
  final KeyedSerialTaskQueue _queue;

  Future<VideoEngagement> setLike(
    NostrEventReference reference,
    VideoLikeIntent intent,
  ) {
    final viewer = _client.publicKeyHex;
    final target = NostrReactionTarget.fromReference(reference);
    final key = NostrLikeMutationKey(viewer, target);
    return _queue.run(key, () => _apply(reference, key, intent));
  }

  Future<VideoEngagement> _apply(
    NostrEventReference reference,
    NostrLikeMutationKey key,
    VideoLikeIntent intent,
  ) async {
    final current = await _viewerState(reference, key, intent);
    return switch (intent) {
      VideoLikeIntent.like => _like(reference, key, current),
      VideoLikeIntent.unlike => _unlike(key, current),
    };
  }

  // A like can proceed on session-local evidence when relays cannot be read;
  // an unlike must know which reaction ids to delete, so without journal
  // evidence the read failure stands.
  Future<NostrViewerReactionState> _viewerState(
    NostrEventReference reference,
    NostrLikeMutationKey key,
    VideoLikeIntent intent,
  ) async {
    try {
      return await _reader.loadViewerState(reference, key);
    } on AppFailure {
      final journalOnly = _reader.journalOnlyViewerState(key);
      final canRecover =
          intent == VideoLikeIntent.like || journalOnly.reactionIds.isNotEmpty;
      if (!canRecover) rethrow;
      return journalOnly;
    }
  }

  Future<VideoEngagement> _like(
    NostrEventReference reference,
    NostrLikeMutationKey key,
    NostrViewerReactionState current,
  ) async {
    if (current.reactionIds.isNotEmpty) return current.engagement;
    final id = await _client.publish(
      _reaction(reference),
      expectedAuthor: key.viewer,
    );
    _journal.recordReaction(key, id);
    return VideoEngagement(
      likeCount: current.engagement.likeCount + 1,
      viewerHasLiked: true,
    );
  }

  Future<VideoEngagement> _unlike(
    NostrLikeMutationKey key,
    NostrViewerReactionState current,
  ) async {
    if (current.reactionIds.isEmpty) return current.engagement;
    final deletionId = await _client.publish(
      _deletion(current.reactionIds),
      expectedAuthor: key.viewer,
    );
    _journal.recordDeletion(key, deletionId, current.reactionIds);
    return VideoEngagement(
      likeCount: current.engagement.likeCount - 1,
      viewerHasLiked: false,
    );
  }
}

NostrUnsignedEvent _reaction(NostrEventReference reference) {
  return NostrUnsignedEvent(
    kind: 7,
    tags: <List<String>>[
      <String>['e', reference.eventId],
      <String>['p', reference.authorPublicKeyHex],
      <String>['k', '${reference.kind}'],
      if (reference.identifier case final identifier?)
        <String>[
          'a',
          '${reference.kind}:${reference.authorPublicKeyHex}:$identifier',
        ],
    ],
    content: '+',
  );
}

NostrUnsignedEvent _deletion(Iterable<NostrEventId> reactionIds) {
  return NostrUnsignedEvent(
    kind: 5,
    tags: <List<String>>[
      for (final reactionId in reactionIds) <String>['e', reactionId],
      const <String>['k', '7'],
    ],
    content: 'Removed like',
  );
}
