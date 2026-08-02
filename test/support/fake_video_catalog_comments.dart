import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/comments/domain/video_comment.dart';
import 'package:ghostr/features/comments/domain/video_comments_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import 'fake_video_catalog_base.dart';
import 'nostr_test_values.dart';

mixin FakeVideoCatalogComments on FakeVideoCatalogBase
    implements VideoCommentsRepository {
  Map<String, List<VideoComment>> get commentsByPost;
  AppFailure? get commentsFailure;
  Future<List<VideoComment>>? get commentsResponse;

  int commentLoadCount = 0;

  @override
  Future<List<VideoComment>> loadComments(VideoPost post) async {
    commentLoadCount += 1;
    if (commentsFailure case final AppFailure failure) {
      throw failure;
    }
    if (commentsResponse case final Future<List<VideoComment>> response) {
      return response;
    }
    return <VideoComment>[...commentsByPost[post.id] ?? <VideoComment>[]];
  }

  @override
  Future<VideoComment> publishComment({
    required VideoPost post,
    required String content,
    VideoComment? replyTo,
  }) async {
    final comments = commentsByPost.putIfAbsent(post.id, () => []);
    final comment = VideoComment(
      identity: VideoCommentIdentity.parse(
        id: publishedEventId(comments.length + 1),
        authorPublicKeyHex: testViewerPublicKey,
      ),
      text: VideoCommentText(authorLabel: 'You', content: content),
      createdAt: DateTime.now(),
      parentCommentId: replyTo?.id,
    );
    comments.add(comment);
    return comment;
  }
}
