import 'package:ghostr/features/comments/domain/nostr_comments_port.dart';
import 'package:ghostr/features/comments/domain/video_comments_repository.dart';
import 'package:ghostr/features/engagement/domain/nostr_engagement_port.dart';
import 'package:ghostr/features/engagement/domain/video_engagement_repository.dart';
import 'package:ghostr/features/publish/domain/video_publishing_repository.dart';
import 'package:ghostr/features/social/data/social_graph_cache.dart';
import 'package:ghostr/features/social/domain/nostr_social_port.dart';
import 'package:ghostr/features/video_catalog/data/hybrid_video_comments_repository.dart';
import 'package:ghostr/features/video_catalog/data/hybrid_video_engagement_repository.dart';
import 'package:ghostr/features/video_catalog/data/hybrid_video_publishing_repository.dart';
import 'package:ghostr/features/video_catalog/domain/aggregating_video_profile_repository.dart';
import 'package:ghostr/features/video_catalog/domain/filtered_video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/filtered_video_search_repository.dart';
import 'package:ghostr/features/video_catalog/data/local_video_store.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/hybrid_video_reader.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_video_interactions.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/video_profile_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_repository.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'fakes.dart';

class HybridHarnessPorts {
  const HybridHarnessPorts({this.social, this.engagement, this.comments});

  final NostrSocialPort? social;
  final NostrEngagementPort? engagement;
  final NostrCommentsPort? comments;
}

class HybridRepositoryHarness {
  const HybridRepositoryHarness({
    required this.feed,
    required this.profile,
    required this.search,
    required this.publishing,
    required this.engagement,
    required this.comments,
    required this.localStore,
    required this.failureReporter,
  });

  final VideoFeedRepository feed;
  final VideoProfileRepository profile;
  final VideoSearchRepository search;
  final VideoPublishingRepository publishing;
  final VideoEngagementRepository engagement;
  final VideoCommentsRepository comments;
  final LocalVideoStore localStore;
  final RecordingFailureReporter failureReporter;
}

Future<HybridRepositoryHarness> buildHybridRepositoryHarness(
  RemoteVideoSource remote, {
  HybridHarnessPorts ports = const HybridHarnessPorts(),
}) async {
  SharedPreferences.setMockInitialValues({});
  final local = LocalVideoStore(
    await SharedPreferences.getInstance(),
    accountScope: testAccountStorageScope(),
  );
  final reporter = RecordingFailureReporter();
  final social = SocialGraphCache(
    ports.social ?? FakeNostrSocialPort(),
    local,
    reporter,
  );
  final interactions = NostrVideoInteractions(
    ports.engagement ?? FakeNostrEngagementPort(),
    ports.comments ?? FakeNostrCommentsPort(),
    reporter,
  );
  final reader = HybridVideoReader(
    remote: remote,
    local: local,
    interactions: interactions,
    failureReporter: reporter,
  );
  return HybridRepositoryHarness(
    feed: FilteredVideoFeedRepository(reader, social),
    profile: AggregatingVideoProfileRepository(reader, social),
    search: FilteredVideoSearchRepository(reader, social),
    publishing: HybridVideoPublishingRepository(
      local,
      FakeNostrVideoPublisherPort(),
      reporter,
    ),
    engagement: HybridVideoEngagementRepository(interactions),
    comments: HybridVideoCommentsRepository(interactions),
    localStore: local,
    failureReporter: reporter,
  );
}
