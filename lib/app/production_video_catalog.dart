import 'package:ghostr/app/production_nostr_services.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/app/video_catalog_services.dart';
import 'package:ghostr/app/video_feed_binding.dart';
import 'package:ghostr/core/storage/account_storage_scope.dart';
import 'package:ghostr/features/comments/data/nostr_comments_repository.dart';
import 'package:ghostr/features/engagement/data/nostr_engagement_repository.dart';
import 'package:ghostr/features/settings/data/local_app_settings_repository.dart';
import 'package:ghostr/features/social/data/social_graph_cache.dart';
import 'package:ghostr/features/video_catalog/data/hybrid_video_comments_repository.dart';
import 'package:ghostr/features/video_catalog/data/hybrid_video_engagement_repository.dart';
import 'package:ghostr/features/video_catalog/data/hybrid_video_publishing_repository.dart';
import 'package:ghostr/features/video_catalog/data/local_video_store.dart';
import 'package:ghostr/features/video_catalog/data/nostr_creator_search_source.dart';
import 'package:ghostr/features/video_catalog/data/recent_videos_trending_hashtags.dart';
import 'package:ghostr/features/video_catalog/domain/aggregating_video_profile_repository.dart';
import 'package:ghostr/features/video_catalog/domain/discovery_video_search_repository.dart';
import 'package:ghostr/features/video_catalog/domain/filtered_video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/hybrid_video_reader.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_video_interactions.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_feed_updates.dart';
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
) => _ProductionVideoCatalog(inputs).build();

final class _ProductionVideoCatalog {
  _ProductionVideoCatalog(this.inputs);

  final ProductionVideoCatalogInputs inputs;
  static const _reporter = DeveloperFailureReporter();

  late final _delivery = inputs.delivery;
  late final _nostr = inputs.nostr;
  late final _local = LocalVideoStore(
    inputs.preferences,
    accountScope: inputs.accountScope,
  );
  late final _social = SocialGraphCache(
    _nostr.adapters.social,
    _local,
    _reporter,
  );
  late final _interactions = NostrVideoInteractions(
    NostrEngagementRepository(_nostr.eventClient),
    NostrCommentsRepository(_nostr.eventClient),
    _reporter,
  );
  late final _reader = HybridVideoReader(
    remote: _delivery.remoteSource,
    local: _local,
    interactions: _interactions,
    failureReporter: _reporter,
  );
  late final _feed = WatchAwareVideoFeedRepository(
    feed: FilteredVideoFeedRepository(_reader, _social),
    history: inputs.watchHistory,
    settings: inputs.settingsRepository,
    failureReporter: _reporter,
  );
  late final _search = DiscoveryVideoSearchRepository(
    videos: _delivery.searchSource,
    creators: NostrCreatorSearchSource(_nostr.eventClient),
    social: _social,
    failureReporter: _reporter,
  );

  VideoCatalogServices build() {
    return VideoCatalogServices(
      feed: VideoFeedBinding(
        repository: _feed,
        updates: _delivery.playbackCapabilities.supportsAny
            ? RemoteVideoFeedUpdates(
                remote: _delivery.remoteSource,
                social: _social,
              )
            : null,
      ),
      engagement: HybridVideoEngagementRepository(_interactions),
      profile: AggregatingVideoProfileRepository(_reader, _social),
      search: _search,
      searchUpdates: _search,
      trending: RecentVideosTrendingHashtags(_delivery.discoverySource),
      publishing: HybridVideoPublishingRepository(
        _local,
        _nostr.publisher,
        _reporter,
      ),
      comments: HybridVideoCommentsRepository(_interactions),
      social: _social,
    );
  }
}
