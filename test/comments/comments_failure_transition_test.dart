import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/comments/domain/video_comment.dart';
import 'package:ghostr/features/comments/presentation/comments_state.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('state transitions cannot erase a failure message', () {
    const state = CommentsState.failure('Relay unavailable');
    final comment = VideoComment(
      identity: VideoCommentIdentity.parse(
        id: testEventId,
        authorPublicKeyHex: testCreatorPublicKey,
      ),
      text: VideoCommentText(authorLabel: 'Nora', content: 'Reply'),
      createdAt: DateTime.utc(2026, 8, 2),
    );
    final changed = [
      state.withReply(comment),
      state.posting(),
      state.withNotice('Try again'),
    ];

    expect(changed.map((item) => item.status),
        everyElement(CommentsStatus.failure));
    expect(
        changed.map((item) => item.message), everyElement('Relay unavailable'));
  });
}
