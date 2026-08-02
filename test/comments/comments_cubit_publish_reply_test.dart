import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/comments/domain/video_comment.dart';
import 'package:ghostr/features/comments/presentation/comments_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/nostr_test_values.dart';

void main() {
  final parent = VideoComment(
    identity: VideoCommentIdentity.parse(
      id: testEventId,
      authorPublicKeyHex: testCreatorPublicKey,
    ),
    text: VideoCommentText(authorLabel: 'Alice', content: 'Parent'),
    createdAt: DateTime(2026, 8, 2),
  );
  blocTest<CommentsCubit, CommentsState>(
    'loads comments, selects a parent, and publishes a reply',
    build: () => CommentsCubit(
      FakeVideoCatalogRepository(
        forYouFeed: [samplePost()],
        comments: FakeCommentsScenario(commentsByPost: {
          'post-1': [parent],
        }),
      ),
      samplePost(),
    ),
    act: (cubit) async {
      await cubit.load();
      cubit.selectReply(parent);
      await cubit.publish('A reply');
    },
    verify: (cubit) {
      expect(cubit.state.comments.map((item) => item.content), [
        'Parent',
        'A reply',
      ]);
      expect(cubit.state.replyTo, isNull);
      expect(cubit.state.status, CommentsStatus.ready);
    },
  );
}
