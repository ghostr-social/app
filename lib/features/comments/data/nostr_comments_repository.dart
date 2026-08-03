import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_deletion_lookup.dart';
import 'package:ghostr/core/nostr/nostr_event_client.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_fair_query.dart';
import 'package:ghostr/core/time/clock.dart';
import 'package:ghostr/features/comments/data/nostr_comment_query_batch.dart';
import 'package:ghostr/features/comments/domain/nostr_comments_port.dart';
import 'package:ghostr/features/comments/domain/video_comment.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';

class NostrCommentsRepository implements NostrCommentsPort {
  const NostrCommentsRepository(
    this._client, {
    Clock clock = systemClock,
    Duration hydrationTimeout = nostrHydrationDeadline,
  })  : _clock = clock,
        _hydrationTimeout = hydrationTimeout;

  final NostrEventClient _client;
  final Clock _clock;
  final Duration _hydrationTimeout;

  @override
  Future<List<VideoComment>> load(NostrEventReference reference) async {
    final comments = await loadBatch(<NostrEventReference>[reference]);
    return comments[reference.eventId]!;
  }

  @override
  Future<Map<NostrEventId, List<VideoComment>>> loadBatch(
    List<NostrEventReference> references,
  ) async {
    final unique = _uniqueReferences(references);
    if (unique.isEmpty) return const <NostrEventId, List<VideoComment>>{};
    final budget = NostrQueryBudget(_hydrationTimeout);
    final events = await loadNostrCommentEvents(_client, unique, budget);
    final groups = unique.map((reference) {
      return _commentEventsFor(events, reference);
    }).toList(growable: false);
    final deletedIds = await loadGroupedAuthorValidNostrDeletionIds(
      _client,
      groups,
      budget: budget,
    );
    return Map<NostrEventId, List<VideoComment>>.unmodifiable({
      for (final reference in unique)
        reference.eventId: _commentsFor(events, deletedIds, reference),
    });
  }

  List<VideoComment> _commentsFor(
    List<NostrEventRecord> events,
    Set<NostrEventId> deletedIds,
    NostrEventReference reference,
  ) {
    final matching = _commentEventsFor(events, reference).where((event) {
      return !deletedIds.contains(event.id);
    });
    final comments = matching.where(_hasContent).map((event) {
      return _toComment(event);
    }).toList();
    comments.sort((left, right) => left.createdAt.compareTo(right.createdAt));
    return comments;
  }

  List<NostrEventRecord> _commentEventsFor(
    List<NostrEventRecord> events,
    NostrEventReference reference,
  ) {
    final name = nostrCommentRootTagName(reference);
    final value = nostrCommentRootValue(reference);
    return events.where((event) {
      return event.tagValues(name).contains(value);
    }).toList(growable: false);
  }

  List<NostrEventReference> _uniqueReferences(
    List<NostrEventReference> references,
  ) {
    return <NostrEventId, NostrEventReference>{
      for (final reference in references) reference.eventId: reference,
    }.values.take(maxNostrTargetsPerFamily).toList(growable: false);
  }

  @override
  Future<VideoComment> publish({
    required NostrEventReference reference,
    required String content,
    VideoComment? replyTo,
  }) async {
    final normalized = content.trim();
    if (normalized.isEmpty) {
      throw const AppFailure('Write a comment before posting.');
    }
    final authorPublicKeyHex = _client.publicKeyHex;
    final id = await _client.publish(
      NostrUnsignedEvent(
        kind: 1111,
        tags: _commentTags(reference, replyTo),
        content: normalized,
      ),
      expectedAuthor: authorPublicKeyHex,
    );
    return _publishedComment(id, normalized, authorPublicKeyHex, replyTo);
  }

  VideoComment _publishedComment(
    NostrEventId id,
    String content,
    NostrPublicKeyHex authorPublicKeyHex,
    VideoComment? replyTo,
  ) {
    return VideoComment(
      identity: VideoCommentIdentity.parse(
        id: id,
        authorPublicKeyHex: authorPublicKeyHex,
      ),
      text: VideoCommentText(
        authorLabel: _authorLabel(authorPublicKeyHex),
        content: content,
      ),
      createdAt: _clock(),
      parentCommentId: replyTo?.id,
    );
  }

  List<List<String>> _commentTags(
    NostrEventReference reference,
    VideoComment? replyTo,
  ) {
    return <List<String>>[
      <String>[
        nostrCommentRootTagName(reference),
        nostrCommentRootValue(reference),
      ],
      <String>['K', '${reference.kind}'],
      <String>['P', reference.authorPublicKeyHex],
      ..._parentTags(reference, replyTo),
    ];
  }

  List<List<String>> _parentTags(
    NostrEventReference reference,
    VideoComment? replyTo,
  ) {
    if (replyTo != null) {
      return <List<String>>[
        <String>['e', replyTo.id],
        const <String>['k', '1111'],
        <String>['p', replyTo.authorPublicKeyHex],
      ];
    }
    return <List<String>>[
      <String>[
        nostrCommentRootTagName(reference).toLowerCase(),
        nostrCommentRootValue(reference),
      ],
      if (reference.identifier != null) <String>['e', reference.eventId],
      <String>['k', '${reference.kind}'],
      <String>['p', reference.authorPublicKeyHex],
    ];
  }

  VideoComment _toComment(NostrEventRecord event) {
    return VideoComment(
      identity: VideoCommentIdentity.parse(
        id: event.id,
        authorPublicKeyHex: event.authorPublicKeyHex,
      ),
      text: VideoCommentText(
        authorLabel: _authorLabel(event.authorPublicKeyHex),
        content: event.content.trim(),
      ),
      createdAt: DateTime.fromMillisecondsSinceEpoch(
        event.createdAt * 1000,
        isUtc: true,
      ),
      parentCommentId: _parentCommentId(event),
    );
  }

  NostrEventId? _parentCommentId(NostrEventRecord event) {
    if (!event.tagValues('k').contains('1111')) return null;
    final id = event.tagValues('e').firstOrNull;
    if (id == null) return null;
    try {
      return NostrEventId.parse(id);
    } on FormatException {
      return null;
    }
  }

  String _authorLabel(String publicKey) {
    return publicKey.length > 12 ? '${publicKey.substring(0, 12)}…' : publicKey;
  }

  bool _hasContent(NostrEventRecord event) => event.content.trim().isNotEmpty;
}
