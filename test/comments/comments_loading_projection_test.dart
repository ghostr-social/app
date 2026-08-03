import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/comments/domain/video_comment.dart';
import 'package:ghostr/features/comments/presentation/comments_state.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('loading comments expose inert content and transitions', () {
    final state = CommentsState.loading();
    final comment = VideoComment(
      identity: VideoCommentIdentity.parse(
        id: testEventId,
        authorPublicKeyHex: testCreatorPublicKey,
      ),
      text: VideoCommentText(authorLabel: 'Nora', content: 'Hello'),
      createdAt: DateTime.utc(2026, 8, 3),
    );

    expect(state.status, CommentsStatus.loading);
    expect(state.comments, isEmpty);
    expect(state.replyTo, isNull);
    expect(state.isPosting, isFalse);
    expect(state.message, isNull);
    expect(state.published(comment), same(state));
    expect(state.withNotice('offline'), same(state));
    expect(state.withoutNotice(), same(state));
  });
}
