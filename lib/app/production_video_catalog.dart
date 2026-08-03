import 'package:ghostr/app/production_nostr_services.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/app/video_catalog_services.dart';
import 'package:ghostr/core/nostr/scheduled_nostr_event_client.dart';
import 'package:ghostr/core/storage/account_storage_scope.dart';
import 'package:ghostr/features/comments/data/nostr_comments_repository.dart';
import 'package:ghostr/features/engagement/data/nostr_engagement_repository.dart';
import 'package:ghostr/features/settings/data/local_app_settings_repository.dart';
import 'package:ghostr/features/social/data/social_graph_cache.dart';
import 'package:ghostr/features/video_catalog/data/hybrid_video_comments_repository.dart';
import 'package:ghostr/features/video_catalog/data/hybrid_video_engagement_repository.dart';
import 'package:ghostr/features/video_catalog/data/hybrid_video_publishing_repository.dart';
import 'package:ghostr/features/video_catalog/data/local_video_store.dart';
import 'package:ghostr/features/video_catalog/data/metadata_creator_search_source.dart';
import 'package:ghostr/features/video_catalog/data/recent_videos_trending_hashtags.dart';
import 'package:ghostr/features/video_catalog/data/scheduled_creator_search_source.dart';
import 'package:ghostr/features/video_catalog/domain/aggregating_video_profile_repository.dart';
import 'package:ghostr/features/video_catalog/domain/discovery_video_search_repository.dart';
import 'package:ghostr/features/video_catalog/domain/filtered_video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/hybrid_video_reader.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_video_interactions.dart';
import 'package:ghostr/features/watch_history/data/local_watch_history_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_feed_repository.dart';
import 'package:ghostr/platform/logging/developer_failure_reporter.dart';
import 'package:shared_preferences/shared_preferences.dart';

class ProductionVideoCatalogInputs {
  const ProductionVideoCatalogInputs({
    required this.preferences,
    required this.delivery,
    required this.nostr,
    required this.accountScope,
    required this.watchHistory,
    required this.settingsRepository,
  });

  final SharedPreferences preferences;
  final ProductionVideoDelivery delivery;
  final ProductionNostrServices nostr;
  final AccountStorageScope accountScope;
  final LocalWatchHistoryRepository watchHistory;
  final LocalAppSettingsRepository settingsRepository;
}

VideoCatalogServices buildProductionVideoCatalog(
  ProductionVideoCatalogInputs inputs,
) {
  final delivery = inputs.delivery;
  final nostr = inputs.nostr;
  final local = LocalVideoStore(
    inputs.preferences,
    accountScope: inputs.accountScope,
  );
  const reporter = DeveloperFailureReporter();
  final social = SocialGraphCache(nostr.adapters.social, local, reporter);
  // Engagement reads queue behind the shared pool so like/comment counts
  // never crowd out what the viewer is actively looking at.
  final scheduledEvents = ScheduledNostrEventClient(
    client: nostr.eventClient,
    scheduler: delivery.scheduler,
  );
  final interactions = NostrVideoInteractions(
    NostrEngagementRepository(scheduledEvents),
    NostrCommentsRepository(scheduledEvents),
    reporter,
  );
  final reader = HybridVideoReader(
    remote: delivery.remoteSource,
    local: local,
    interactions: interactions,
    failureReporter: reporter,
  );
  return VideoCatalogServices(
    feed: WatchAwareVideoFeedRepository(
      feed: FilteredVideoFeedRepository(reader, social),
      history: inputs.watchHistory,
      settings: inputs.settingsRepository,
      failureReporter: reporter,
    ),
    engagement: HybridVideoEngagementRepository(interactions),
    profile: AggregatingVideoProfileRepository(reader, social),
    search: DiscoveryVideoSearchRepository(
      videos: delivery.searchSource,
      creators: ScheduledCreatorSearchSource(
        source: MetadataCreatorSearchSource(nostr.profileSearch),
        scheduler: delivery.scheduler,
      ),
      social: social,
      failureReporter: reporter,
    ),
    trending: RecentVideosTrendingHashtags(
      delivery.discoverySource,
      delivery.scheduler,
    ),
    publishing: HybridVideoPublishingRepository(
      local,
      nostr.publisher,
      reporter,
    ),
    comments: HybridVideoCommentsRepository(interactions),
    social: social,
  );
}
