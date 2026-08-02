import 'package:ghostr/features/comments/domain/video_comment.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

abstract interface class VideoCommentsRepository {
  Future<List<VideoComment>> loadComments(VideoPost post);

  Future<VideoComment> publishComment({
    required VideoPost post,
    required String content,
    VideoComment? replyTo,
  });
}
