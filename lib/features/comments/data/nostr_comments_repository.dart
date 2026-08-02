import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_client.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/time/clock.dart';
import 'package:ghostr/features/comments/domain/nostr_comments_port.dart';
import 'package:ghostr/features/comments/domain/video_comment.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';

class NostrCommentsRepository implements NostrCommentsPort {
  const NostrCommentsRepository(
    this._client, {
    Clock clock = systemClock,
  }) : _clock = clock;

  final NostrEventClient _client;
  final Clock _clock;

  @override
  Future<List<VideoComment>> load(NostrEventReference reference) async {
    final events = await _client.query(NostrEventQuery(
      kinds: const <int>[1111],
      tagFilters: <NostrTagFilter>[
        NostrTagFilter(
          name: _rootTagName(reference),
          values: <String>[_rootValue(reference)],
        ),
      ],
    ));
    final comments = events.where(_hasContent).map((event) {
      return _toComment(event, reference);
    }).toList();
    comments.sort((left, right) => left.createdAt.compareTo(right.createdAt));
    return comments;
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
    final id = await _client.publish(NostrUnsignedEvent(
      kind: 1111,
      tags: _commentTags(reference, replyTo),
      content: normalized,
    ));
    return _publishedComment(id, normalized, replyTo);
  }

  VideoComment _publishedComment(
    NostrEventId id,
    String content,
    VideoComment? replyTo,
  ) {
    return VideoComment(
      identity: VideoCommentIdentity.parse(
        id: id,
        authorPublicKeyHex: _client.publicKeyHex,
      ),
      text: VideoCommentText(
        authorLabel: _authorLabel(_client.publicKeyHex),
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
      <String>[_rootTagName(reference), _rootValue(reference)],
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
      <String>[_rootTagName(reference).toLowerCase(), _rootValue(reference)],
      if (reference.identifier != null) <String>['e', reference.eventId],
      <String>['k', '${reference.kind}'],
      <String>['p', reference.authorPublicKeyHex],
    ];
  }

  VideoComment _toComment(
    NostrEventRecord event,
    NostrEventReference reference,
  ) {
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
    return id == null ? null : NostrEventId.parse(id);
  }

  String _rootTagName(NostrEventReference reference) {
    return reference.identifier == null ? 'E' : 'A';
  }

  String _rootValue(NostrEventReference reference) {
    final identifier = reference.identifier;
    if (identifier == null) return reference.eventId;
    return '${reference.kind}:${reference.authorPublicKeyHex}:$identifier';
  }

  String _authorLabel(String publicKey) {
    return publicKey.length > 12 ? '${publicKey.substring(0, 12)}…' : publicKey;
  }

  bool _hasContent(NostrEventRecord event) => event.content.trim().isNotEmpty;
}
