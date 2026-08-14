import 'package:ghostr/app/production_nostr_services.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/app/video_catalog_services.dart';
import 'package:ghostr/app/video_feed_binding.dart';
import 'package:ghostr/core/storage/account_storage_scope.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/comments/data/nostr_comments_repository.dart';
import 'package:ghostr/features/engagement/data/nostr_engagement_repository.dart';
import 'package:ghostr/features/reposts/data/nostr_video_repost_repository.dart';
import 'package:ghostr/features/reposts/data/nostr_author_write_relay_lookup.dart';
import 'package:ghostr/features/settings/data/local_app_settings_repository.dart';
import 'package:ghostr/features/social/data/social_graph_cache.dart';
import 'package:ghostr/features/video_catalog/data/hybrid_video_comments_repository.dart';
import 'package:ghostr/features/video_catalog/data/hybrid_video_engagement_repository.dart';
import 'package:ghostr/features/video_catalog/data/hybrid_video_publishing_repository.dart';
import 'package:ghostr/features/video_catalog/data/local_video_store.dart';
import 'package:ghostr/features/video_catalog/data/nostr_creator_search_source.dart';
import 'package:ghostr/features/video_catalog/data/recent_videos_trending_hashtags.dart';
import 'package:ghostr/features/video_catalog/domain/aggregating_video_profile_repository.dart';
import 'package:ghostr/features/video_catalog/domain/account_scoped_video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/discovery_video_search_repository.dart';
import 'package:ghostr/features/video_catalog/domain/filtered_video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/following_feed_scope.dart';
import 'package:ghostr/features/video_catalog/domain/hybrid_video_reader.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_video_interactions.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_feed_updates.dart';
import 'package:ghostr/features/video_catalog/domain/repost_hydrated_video_feed_repository.dart';
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
  late final _repostRelayHint = NostrAuthorWriteRelayLookup(_nostr.eventClient);
  late final _reposts = NostrVideoRepostRepository(
    _nostr.eventClient,
    relayHint: _repostRelayHint.call,
    hydrationTimeout: Duration.zero,
  );
  late final _followingScopes = FollowingFeedScopeReader(
    _social,
    _activeViewer,
  );
  late final _feed = AccountScopedVideoFeedRepository(
    RepostHydratedVideoFeedRepository(
      WatchAwareVideoFeedRepository(
        feed: FilteredVideoFeedRepository(
          _reader,
          _social,
          followingScopes: _followingScopes,
        ),
        history: inputs.watchHistory,
        settings: inputs.settingsRepository,
        failureReporter: _reporter,
      ),
      _reposts,
    ),
    _activeViewer,
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
                followingScopes: _followingScopes,
              )
            : null,
      ),
      discovery: VideoCatalogDiscoveryServices(
        profile: AggregatingVideoProfileRepository(_reader, _social),
        search: _search,
        searchUpdates: _search,
        trending: RecentVideosTrendingHashtags(_delivery.discoverySource),
      ),
      interactions: VideoCatalogInteractionServices(
        engagement: HybridVideoEngagementRepository(_interactions),
        comments: HybridVideoCommentsRepository(_interactions),
        social: _social,
        reposts: _reposts,
      ),
      authoring: VideoCatalogAuthoringServices(
        publishing: HybridVideoPublishingRepository(
          _local,
          _nostr.publisher,
          _reporter,
        ),
      ),
    );
  }

  NostrPublicKeyHex? _activeViewer() {
    try {
      return _nostr.eventClient.publicKeyHex;
    } on AppFailure {
      return null;
    }
  }
}
