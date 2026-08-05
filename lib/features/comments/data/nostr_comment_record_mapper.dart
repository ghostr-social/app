part of 'nostr_comments_repository.dart';

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
