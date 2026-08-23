import 'package:ghostr/features/comments/domain/video_comment.dart';
import 'package:ghostr/features/comments/domain/video_comments_repository.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/engagement/domain/video_engagement_repository.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_origin.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_workflow.dart';

final class WarpRemoteFeedRepository implements VideoFeedRepository {
  const WarpRemoteFeedRepository(this._source);

  final RemoteVideoSource _source;

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) => _source.loadRemoteFeed();

  @override
  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched = false,
  }) async {
    final posts = await _source.loadRemoteFeed(olderThan: olderThan);
    return VideoFeedPage(posts: posts);
  }
}

final class WarpNoopEngagement implements VideoEngagementRepository {
  const WarpNoopEngagement();

  @override
  Future<VideoPost> toggleLike(VideoPost post) async => post;
}

final class WarpNoopComments implements VideoCommentsRepository {
  const WarpNoopComments();

  @override
  Future<List<VideoComment>> loadComments(VideoPost post) async => const [];

  @override
  Future<VideoComment> publishComment({
    required VideoPost post,
    required String content,
    VideoComment? replyTo,
  }) => throw UnsupportedError('Comments are outside this WARP journey.');
}

final class WarpNoopShare implements VideoShareWorkflow {
  const WarpNoopShare();

  @override
  bool supports(VideoMediaSource media) => false;

  @override
  Future<void> share(
    VideoMediaSource media, {
    required VideoShareOrigin origin,
  }) async {}
}
