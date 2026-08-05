import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/trending_hashtags.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_profile_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

mixin FakeVideoCatalogDiscoveryBehavior
    implements
        VideoProfileRepository,
        VideoSearchRepository,
        VideoSearchUpdates,
        TrendingHashtagsSource,
        SocialGraphRepository {
  Map<String, ProfileDetails> get profiles;
  Set<ProfileId> get blockedProfiles;
  Set<ProfileId> get followedProfiles;
  List<VideoPost> get searchResults;
  List<String> get searchQueries;
  List<String> get creatorQueries;
  List<ProfileSummary> get creatorResults;
  List<String> get trendingTags;

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() async {
    return Set<ProfileId>.of(followedProfiles);
  }

  @override
  Future<Set<ProfileId>> loadBlockedProfiles() async {
    return Set<ProfileId>.of(blockedProfiles);
  }

  @override
  Future<ProfileDetails> loadProfile(
    ProfileSummary viewer,
    ProfileId profileId,
  ) async {
    final fallback = ProfileDetails.empty(ProfileSummary.unknown(profileId));
    final details = profiles[profileId] ?? fallback;
    return details.copyWith(isBlocked: blockedProfiles.contains(profileId));
  }

  @override
  Future<VideoFeedPage> searchVideos(
    String query, {
    DateTime? olderThan,
  }) async {
    searchQueries.add(query);
    final posts = olderThan == null ? searchResults : const <VideoPost>[];
    return VideoFeedPage(posts: _visible(posts));
  }

  @override
  Future<VideoFeedPage> loadMoreVideos(String query) async {
    return VideoFeedPage(posts: const <VideoPost>[]);
  }

  @override
  Future<List<ProfileSummary>> searchCreators(String query) async {
    creatorQueries.add(query);
    return creatorResults;
  }

  @override
  Stream<VideoSearchSnapshot> watchVideos(String query) => const Stream.empty();

  @override
  Future<List<String>> trendingHashtags() async => trendingTags;

  @override
  Future<bool> toggleFollow(ProfileId profileId) async => true;

  @override
  Future<bool> toggleBlock(ProfileId profileId) async {
    if (blockedProfiles.remove(profileId)) return false;
    blockedProfiles.add(profileId);
    return true;
  }

  List<VideoPost> _visible(List<VideoPost> posts) {
    return posts
        .where((post) => !blockedProfiles.contains(post.creator.id))
        .toList();
  }
}
