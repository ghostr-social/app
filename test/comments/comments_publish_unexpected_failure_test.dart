import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/comments/domain/video_comment.dart';
import 'package:ghostr/features/comments/presentation/comments_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('uses an app-safe notice for an unexpected comment publishing error',
      () async {
    final cubit = CommentsCubit(_UnexpectedPublishRepository(), samplePost());
    addTearDown(cubit.close);
    await cubit.load();

    final published = await cubit.publish('A comment');

    expect(published, isFalse);
    expect(cubit.state.notice, 'Could not publish this comment.');
  });
}

class _UnexpectedPublishRepository extends FakeVideoCatalogRepository {
  _UnexpectedPublishRepository() : super(forYouFeed: []);

  @override
  Future<VideoComment> publishComment({
    required VideoPost post,
    required String content,
    VideoComment? replyTo,
  }) {
    throw StateError('signer unavailable');
  }
}
