import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

mixin FakeVideoCatalogFeedBehavior implements VideoFeedRepository {
  List<VideoPost> get forYouFeed;
  List<VideoPost> get followingFeed;
  AppFailure? get feedFailure;
  Set<ProfileId> get blockedProfiles;
  List<bool> get loadFeedExclusions;
  List<DateTime> get olderFeedRequests;
  List<List<VideoPost>> get olderFeedPages;

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) async {
    loadFeedExclusions.add(excludeWatched);
    if (feedFailure case final AppFailure failure) throw failure;
    final posts = kind == FeedKind.forYou ? forYouFeed : followingFeed;
    return _visible(posts);
  }

  @override
  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched = false,
  }) async {
    olderFeedRequests.add(olderThan);
    if (feedFailure case final AppFailure failure) throw failure;
    final posts = olderFeedPages.isEmpty
        ? const <VideoPost>[]
        : olderFeedPages.removeAt(0);
    return VideoFeedPage(
      posts: _visible(posts),
      nextOlderThan: olderFeedPages.isEmpty ? null : olderThan,
    );
  }

  List<VideoPost> _visible(List<VideoPost> posts) {
    return posts
        .where((post) => !blockedProfiles.contains(post.creator.id))
        .toList();
  }
}
