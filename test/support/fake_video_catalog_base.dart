import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/engagement/domain/video_engagement_repository.dart';
import 'package:ghostr/features/publish/domain/video_publishing_repository.dart';
import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_profile_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_repository.dart';

import 'fake_video_catalog_scenarios.dart';

abstract class FakeVideoCatalogBase
    implements
        VideoFeedRepository,
        VideoEngagementRepository,
        VideoProfileRepository,
        VideoSearchRepository,
        VideoPublishingRepository,
        SocialGraphRepository {
  FakeVideoCatalogBase({
    required this.forYouFeed,
    required FakeFeedScenario feed,
    required FakeWriteScenario writes,
  })  : followingFeed = feed.followingFeed ?? forYouFeed,
        searchResults = feed.searchResults ?? forYouFeed,
        profiles = feed.profiles,
        feedFailure = feed.failure,
        likeFailure = writes.likeFailure,
        publishFailure = writes.publishFailure;

  final List<VideoPost> forYouFeed;
  final List<VideoPost> followingFeed;
  final List<VideoPost> searchResults;
  final Map<String, ProfileDetails> profiles;
  final AppFailure? feedFailure;
  final AppFailure? likeFailure;
  final AppFailure? publishFailure;
  final Set<ProfileId> blockedProfiles = <ProfileId>{};
  final Set<ProfileId> followedProfiles = <ProfileId>{};
  final List<bool> loadFeedExclusions = <bool>[];
  final List<String> searchQueries = <String>[];
  final List<DateTime> olderFeedRequests = <DateTime>[];
  final List<List<VideoPost>> olderFeedPages = <List<VideoPost>>[];

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() async {
    return Set<ProfileId>.of(followedProfiles);
  }

  @override
  Future<Set<ProfileId>> loadBlockedProfiles() async {
    return Set<ProfileId>.of(blockedProfiles);
  }

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) async {
    loadFeedExclusions.add(excludeWatched);
    if (feedFailure case final AppFailure failure) throw failure;
    final posts = kind == FeedKind.forYou ? forYouFeed : followingFeed;
    return posts
        .where((post) => !blockedProfiles.contains(post.creator.id))
        .toList();
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
      posts: posts
          .where((post) => !blockedProfiles.contains(post.creator.id))
          .toList(),
      nextOlderThan: olderFeedPages.isEmpty ? null : olderThan,
    );
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
  Future<List<VideoPost>> search(String query) async {
    searchQueries.add(query);
    return searchResults;
  }

  @override
  Future<bool> toggleFollow(ProfileId profileId) async => true;

  @override
  Future<bool> toggleBlock(ProfileId profileId) async {
    if (blockedProfiles.remove(profileId)) return false;
    blockedProfiles.add(profileId);
    return true;
  }
}
