import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/comments/domain/video_comment.dart';
import 'package:ghostr/features/comments/presentation/comments_state.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('does not expose a mutable comments collection', () {
    final comments = CommentsState.content([
      VideoComment(
        identity: VideoCommentIdentity.parse(
          id: testEventId,
          authorPublicKeyHex: testCreatorPublicKey,
        ),
        text: VideoCommentText(authorLabel: 'Nora', content: 'Hello'),
        createdAt: DateTime.utc(2026, 8, 2),
      ),
    ]);

    expect(() => comments.comments.clear(), throwsUnsupportedError);
  });
}
