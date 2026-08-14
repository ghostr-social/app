import 'package:ghostr/core/async/keyed_serial_task_queue.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_client.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/reposts/data/accepted_nostr_repost_journal.dart';
import 'package:ghostr/features/reposts/data/nostr_repost_event_builder.dart';
import 'package:ghostr/features/reposts/data/nostr_repost_reader.dart';
import 'package:ghostr/features/reposts/data/nostr_repost_target.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';

typedef NostrRepostRelayHint =
    Future<String?> Function(NostrPublicKeyHex originalAuthor);

enum VideoRepostIntent { repost, remove }

final class NostrRepostMutationService {
  const NostrRepostMutationService(
    this._dependencies, {
    required NostrRepostRelayHint relayHint,
  }) : _relayHint = relayHint;

  final NostrRepostMutationDependencies _dependencies;
  final NostrRepostRelayHint _relayHint;

  NostrEventClient get _client => _dependencies.client;
  NostrRepostReader get _reader => _dependencies.reader;
  AcceptedNostrRepostJournal get _journal => _dependencies.journal;
  KeyedSerialTaskQueue get _queue => _dependencies.queue;

  Future<NostrViewerRepostState> setRepost(
    NostrEventReference reference,
    VideoRepostIntent intent,
  ) {
    final viewer = _client.publicKeyHex;
    final key = NostrRepostMutationKey(
      viewer,
      NostrRepostTarget.fromReference(reference),
    );
    return _queue.run(key, () => _apply(reference, key, intent));
  }

  Future<NostrViewerRepostState> _apply(
    NostrEventReference reference,
    NostrRepostMutationKey key,
    VideoRepostIntent intent,
  ) async {
    final current = await _viewerState(reference, key);
    return switch (intent) {
      VideoRepostIntent.repost => _repost(reference, key, current),
      VideoRepostIntent.remove => _remove(reference, key, current),
    };
  }

  Future<NostrViewerRepostState> _viewerState(
    NostrEventReference reference,
    NostrRepostMutationKey key,
  ) async {
    try {
      return await _reader.loadViewerState(reference, key);
    } on AppFailure {
      final journal = _reader.journalOnlyState(key);
      if (!_journal.hasEvidence(key)) rethrow;
      return journal;
    }
  }

  Future<NostrViewerRepostState> _repost(
    NostrEventReference reference,
    NostrRepostMutationKey key,
    NostrViewerRepostState current,
  ) async {
    if (current.viewerHasReposted) return current;
    final event = buildRepostEvent(reference, await _relayHintFor(reference));
    final id = await _client.publish(event, expectedAuthor: key.viewer);
    _journal.recordRepost(key, id);
    _reader.verifyViewer(key.viewer);
    return NostrViewerRepostState(<NostrEventId>{id});
  }

  Future<String?> _relayHintFor(NostrEventReference reference) async {
    try {
      return await _relayHint(reference.authorPublicKeyHex);
    } on AppFailure {
      if (repostKindFor(reference) == 16) return null;
      rethrow;
    }
  }

  Future<NostrViewerRepostState> _remove(
    NostrEventReference reference,
    NostrRepostMutationKey key,
    NostrViewerRepostState current,
  ) async {
    if (current.repostIds.isEmpty) return current;
    final event = buildRepostDeletion(
      current.repostIds,
      repostKindFor(reference),
    );
    await _client.publish(event, expectedAuthor: key.viewer);
    _journal.recordDeletion(key, current.repostIds);
    _reader.verifyViewer(key.viewer);
    return const NostrViewerRepostState({});
  }
}

final class NostrRepostMutationDependencies {
  const NostrRepostMutationDependencies(
    this.client,
    this.reader,
    this.journal,
    this.queue,
  );

  final NostrEventClient client;
  final NostrRepostReader reader;
  final AcceptedNostrRepostJournal journal;
  final KeyedSerialTaskQueue queue;
}
