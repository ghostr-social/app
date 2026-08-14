import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/comments/domain/video_comment.dart';
import 'package:ghostr/features/publish/domain/video_publication.dart';
import 'package:ghostr/features/reposts/domain/video_repost_repository.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_id.dart';

import 'fake_video_catalog_base.dart';
import 'fake_video_catalog_comments.dart';
import 'fake_video_catalog_helpers.dart';
import 'fake_video_catalog_scenarios.dart';

class FakeVideoCatalogRepository extends FakeVideoCatalogBase
    with FakeVideoCatalogComments
    implements VideoRepostRepository {
  FakeVideoCatalogRepository({
    required super.forYouFeed,
    super.feed = const FakeFeedScenario(),
    FakeCommentsScenario comments = const FakeCommentsScenario(),
    super.writes = const FakeWriteScenario(),
    this.cacheStatus = VideoPublicationCacheStatus.stored,
  }) : commentsByPost = {...comments.commentsByPost},
       commentsFailure = comments.failure,
       commentsResponse = comments.response,
       commentPublishBarrier = comments.publishBarrier;

  @override
  final Map<String, List<VideoComment>> commentsByPost;
  @override
  AppFailure? commentsFailure;
  @override
  final Future<List<VideoComment>>? commentsResponse;
  @override
  final Future<void>? commentPublishBarrier;
  final VideoPublicationCacheStatus cacheStatus;
  @override
  Future<VideoPublication> publish({
    required UserSession session,
    required SelectedMedia media,
    required String caption,
  }) async {
    if (publishFailure case final AppFailure failure) throw failure;
    final post = VideoPost(
      identity: VideoPostIdentity(
        id: VideoPostId.parse('local-${forYouFeed.length}'),
        creator: session.profile,
      ),
      content: VideoPostContent(
        caption: caption,
        songName: media.label,
        media: VideoMediaSource.local(media.path),
        publishedAt: DateTime.utc(2026, 3, 12),
      ),
      metrics: VideoPostMetrics(
        likeCount: 0,
        commentCount: 0,
        viewerHasLiked: false,
      ),
    );
    forYouFeed.insert(0, post);
    return VideoPublication(post: post, cacheStatus: cacheStatus);
  }

  @override
  Future<VideoPost> toggleLike(VideoPost post) async {
    if (likeFailure case final AppFailure failure) throw failure;
    final isLiked = !post.viewerHasLiked;
    final updated = post.withInteraction(
      VideoInteractionUpdate(
        likeCount: post.likeCount + (isLiked ? 1 : -1),
        viewerHasLiked: isLiked,
      ),
    );
    replacePost(forYouFeed, updated);
    replacePost(followingFeed, updated);
    return updated;
  }

  @override
  Future<List<VideoPost>> hydrateAll(
    List<VideoPost> posts, {
    VideoRepostHydration mode = VideoRepostHydration.prompt,
  }) async => posts;

  @override
  Future<VideoPost> toggleRepost(VideoPost post) async {
    return post.withRepost(!post.viewerHasReposted);
  }
}
