import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/comments/domain/video_comment.dart';

void main() {
  test('exposes typed comment identity, text, and reply ancestry', () {
    final comment = VideoComment(
      identity: VideoCommentIdentity.parse(
        id: 'a' * 64,
        authorPublicKeyHex: 'b' * 64,
      ),
      text: VideoCommentText(authorLabel: 'Nora', content: 'Hello'),
      createdAt: DateTime.utc(2026, 8, 2),
      parentCommentId: NostrEventId.parse('c' * 64),
    );

    expect(comment.id, 'a' * 64);
    expect(comment.authorPublicKeyHex, 'b' * 64);
    expect(comment.authorLabel, 'Nora');
    expect(comment.content, 'Hello');
    expect(comment.isReply, isTrue);
  });
}
