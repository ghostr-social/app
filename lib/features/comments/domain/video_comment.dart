import 'package:ghostr/core/nostr/nostr_event_identity.dart';

class VideoComment {
  const VideoComment({
    required this.identity,
    required this.text,
    required this.createdAt,
    this.parentCommentId,
  });

  final VideoCommentIdentity identity;
  final VideoCommentText text;
  final DateTime createdAt;
  final NostrEventId? parentCommentId;

  NostrEventId get id => identity.id;

  NostrPublicKeyHex get authorPublicKeyHex => identity.authorPublicKeyHex;

  String get authorLabel => text.authorLabel;

  String get content => text.content;

  bool get isReply => parentCommentId != null;
}

class VideoCommentIdentity {
  factory VideoCommentIdentity.parse({
    required String id,
    required String authorPublicKeyHex,
  }) {
    return VideoCommentIdentity._(
      NostrEventId.parse(id),
      NostrPublicKeyHex.parse(authorPublicKeyHex),
    );
  }

  const VideoCommentIdentity._(this.id, this.authorPublicKeyHex);

  final NostrEventId id;
  final NostrPublicKeyHex authorPublicKeyHex;
}

class VideoCommentText {
  factory VideoCommentText({
    required String authorLabel,
    required String content,
  }) {
    return VideoCommentText._(
      _required(authorLabel, 'Comment author'),
      _required(content, 'Comment content'),
    );
  }

  const VideoCommentText._(this.authorLabel, this.content);

  final String authorLabel;
  final String content;
}

String _required(String raw, String label) {
  final value = raw.trim();
  if (value.isEmpty) throw FormatException('$label is required.');
  return value;
}
