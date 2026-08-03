import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/comments/domain/video_comment.dart';
import 'package:ghostr/features/comments/presentation/comments_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('preserves and clears an app-safe comment publishing notice', () async {
    final cubit = CommentsCubit(_RejectedCommentRepository(), samplePost());
    addTearDown(cubit.close);
    await cubit.load();

    final published = await cubit.publish('A comment');

    expect(published, isFalse);
    expect(cubit.state.notice, 'Relay rejected the comment.');
    expect(cubit.state.isPosting, isFalse);
    await cubit.publish('Try again');
    expect(cubit.state.isPosting, isFalse);
    expect(cubit.state.notice, 'Relay rejected the comment.');
    cubit.clearNotice();
    expect(cubit.state.notice, isNull);
  });
}

class _RejectedCommentRepository extends FakeVideoCatalogRepository {
  _RejectedCommentRepository() : super(forYouFeed: []);

  @override
  Future<VideoComment> publishComment({
    required VideoPost post,
    required String content,
    VideoComment? replyTo,
  }) {
    throw const AppFailure('Relay rejected the comment.');
  }
}
