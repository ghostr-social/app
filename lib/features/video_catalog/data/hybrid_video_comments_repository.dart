import 'package:ghostr/features/comments/domain/video_comment.dart';
import 'package:ghostr/features/comments/domain/video_comments_repository.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_video_interactions.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

class HybridVideoCommentsRepository implements VideoCommentsRepository {
  const HybridVideoCommentsRepository(this._interactions);

  final NostrVideoInteractions _interactions;

  @override
  Future<List<VideoComment>> loadComments(VideoPost post) {
    return _interactions.loadComments(post);
  }

  @override
  Future<VideoComment> publishComment({
    required VideoPost post,
    required String content,
    VideoComment? replyTo,
  }) {
    return _interactions.publishComment(
      post: post,
      content: content,
      replyTo: replyTo,
    );
  }
}
