import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/engagement/domain/video_engagement_repository.dart';
import 'package:ghostr/features/publish/domain/video_publishing_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import 'fake_video_catalog_discovery_behavior.dart';
import 'fake_video_catalog_feed_behavior.dart';
import 'fake_video_catalog_scenarios.dart';

abstract class FakeVideoCatalogBase
    with FakeVideoCatalogFeedBehavior, FakeVideoCatalogDiscoveryBehavior
    implements VideoEngagementRepository, VideoPublishingRepository {
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

  @override
  final List<VideoPost> forYouFeed;
  @override
  final List<VideoPost> followingFeed;
  @override
  final List<VideoPost> searchResults;
  @override
  final Map<String, ProfileDetails> profiles;
  @override
  final AppFailure? feedFailure;
  final AppFailure? likeFailure;
  final AppFailure? publishFailure;
  @override
  final Set<ProfileId> blockedProfiles = <ProfileId>{};
  @override
  final Set<ProfileId> followedProfiles = <ProfileId>{};
  @override
  final List<bool> loadFeedExclusions = <bool>[];
  @override
  final List<String> searchQueries = <String>[];
  @override
  final List<String> creatorQueries = <String>[];
  @override
  final List<ProfileSummary> creatorResults = <ProfileSummary>[];
  @override
  final List<String> trendingTags = <String>[];
  @override
  final List<DateTime> olderFeedRequests = <DateTime>[];
  @override
  final List<List<VideoPost>> olderFeedPages = <List<VideoPost>>[];
}
