import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/failure_reporter.dart';
import 'package:ghostr/features/comments/domain/nostr_comments_port.dart';
import 'package:ghostr/features/comments/domain/video_comment.dart';
import 'package:ghostr/features/engagement/domain/nostr_engagement_port.dart';
import 'package:ghostr/features/engagement/domain/video_engagement.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';
import 'package:ghostr/features/video_catalog/domain/video_like_policy.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

class NostrVideoInteractions {
  const NostrVideoInteractions(
    this._engagement,
    this._comments,
    this._failureReporter, {
    VideoLikePolicy likePolicy = const VideoLikePolicy(),
  }) : _likePolicy = likePolicy;

  final NostrEngagementPort _engagement;
  final NostrCommentsPort _comments;
  final FailureReporter _failureReporter;
  final VideoLikePolicy _likePolicy;

  Future<VideoPost> hydrate(VideoPost post) async {
    final reference = post.nostrReference;
    if (reference == null) return post;
    final engagementFuture = _loadEngagement(reference, post);
    final commentCountFuture = _loadCommentCount(reference, post);
    final engagement = await engagementFuture;
    final commentCount = await commentCountFuture;
    return post.withInteraction(
      likeCount: engagement.likeCount,
      viewerHasLiked: engagement.viewerHasLiked,
      commentCount: commentCount,
    );
  }

  Future<VideoPost> toggleLike(VideoPost post) async {
    final reference = post.nostrReference;
    if (reference == null) return _likePolicy.toggle(post);
    final engagement = await _engagement.toggleLike(reference);
    return post.withInteraction(
      likeCount: engagement.likeCount,
      viewerHasLiked: engagement.viewerHasLiked,
    );
  }

  Future<List<VideoComment>> loadComments(VideoPost post) {
    final reference = post.nostrReference;
    if (reference == null) return Future.value(const <VideoComment>[]);
    return _comments.load(reference);
  }

  Future<VideoComment> publishComment({
    required VideoPost post,
    required String content,
    VideoComment? replyTo,
  }) {
    final reference = post.nostrReference;
    if (reference == null) {
      throw const AppFailure('This video has no Nostr event to comment on.');
    }
    return _comments.publish(
      reference: reference,
      content: content,
      replyTo: replyTo,
    );
  }

  Future<VideoEngagement> _loadEngagement(
    NostrEventReference reference,
    VideoPost fallback,
  ) async {
    try {
      return await _engagement.load(reference);
    } on AppFailure catch (error, stackTrace) {
      _report('NostrVideoInteractions.loadEngagement', error, stackTrace);
      return VideoEngagement(
        likeCount: fallback.likeCount,
        viewerHasLiked: fallback.viewerHasLiked,
      );
    }
  }

  Future<int> _loadCommentCount(
    NostrEventReference reference,
    VideoPost fallback,
  ) async {
    try {
      return (await _comments.load(reference)).length;
    } on AppFailure catch (error, stackTrace) {
      _report('NostrVideoInteractions.loadCommentCount', error, stackTrace);
      return fallback.commentCount;
    }
  }

  void _report(String source, Object error, StackTrace stackTrace) {
    _failureReporter.report(
      source: source,
      error: error,
      stackTrace: stackTrace,
    );
  }
}
