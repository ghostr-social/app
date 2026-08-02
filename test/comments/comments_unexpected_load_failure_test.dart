import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/comments/domain/video_comment.dart';
import 'package:ghostr/features/comments/presentation/comments_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('uses an app-safe message for an unexpected comments load error',
      () async {
    final cubit = CommentsCubit(_UnexpectedLoadRepository(), samplePost());
    addTearDown(cubit.close);

    await cubit.load();

    expect(cubit.state.status, CommentsStatus.failure);
    expect(cubit.state.message, 'Could not load comments from relays.');
  });
}

class _UnexpectedLoadRepository extends FakeVideoCatalogRepository {
  _UnexpectedLoadRepository() : super(forYouFeed: []);

  @override
  Future<List<VideoComment>> loadComments(VideoPost post) {
    throw StateError('relay unavailable');
  }
}
