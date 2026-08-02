import 'package:flutter/widgets.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:ghostr/app/app_dependencies.dart';
import 'package:ghostr/app/production_nostr_services.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/app/video_catalog_services.dart';
import 'package:ghostr/features/activity/data/local_activity_repository.dart';
import 'package:ghostr/features/activity/data/nostr_activity_repository.dart';
import 'package:ghostr/features/comments/data/nostr_comments_repository.dart';
import 'package:ghostr/features/engagement/data/nostr_engagement_repository.dart';
import 'package:ghostr/features/session/data/ndk_nostr_identity_deriver.dart';
import 'package:ghostr/features/session/data/secure_session_repository.dart';
import 'package:ghostr/features/settings/data/local_app_settings_repository.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/social/data/social_graph_cache.dart';
import 'package:ghostr/features/video_catalog/data/hybrid_video_comments_repository.dart';
import 'package:ghostr/features/video_catalog/data/hybrid_video_engagement_repository.dart';
import 'package:ghostr/features/video_catalog/data/hybrid_video_publishing_repository.dart';
import 'package:ghostr/features/video_catalog/data/local_video_store.dart';
import 'package:ghostr/features/video_catalog/domain/aggregating_video_profile_repository.dart';
import 'package:ghostr/features/video_catalog/domain/filtered_video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/filtered_video_search_repository.dart';
import 'package:ghostr/features/video_catalog/domain/hybrid_video_reader.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_video_interactions.dart';
import 'package:ghostr/platform/logging/developer_failure_reporter.dart';
import 'package:ghostr/platform/media/image_picker_media_picker.dart';
import 'package:ghostr/platform/media/inventory_video_playback_port.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/platform/storage/secure_secret_store.dart';
import 'package:image_picker/image_picker.dart';
import 'package:ndk/ndk.dart';
import 'package:shared_preferences/shared_preferences.dart';

export 'production_nostr_services.dart';

typedef PreferencesLoader = Future<SharedPreferences> Function();
typedef ProductionNostrServicesBuilder = ProductionNostrServices Function(
  AppSettings settings,
);
typedef ProductionVideoDeliveryBuilder = Future<ProductionVideoDelivery>
    Function(AppSettings settings, ProductionNostrServices nostr);
typedef ProductionVideoEnvironmentBuilder = ProductionVideoDeliveryEnvironment
    Function(Ndk ndk);

class ProductionDependenciesEnvironment {
  const ProductionDependenciesEnvironment({
    required this.preferencesLoader,
    required this.nostrServicesBuilder,
    required this.videoDeliveryBuilder,
  });

  factory ProductionDependenciesEnvironment.production({
    ProductionVideoEnvironmentBuilder videoEnvironmentBuilder =
        ProductionVideoDeliveryEnvironment.production,
  }) {
    return ProductionDependenciesEnvironment(
      preferencesLoader: SharedPreferences.getInstance,
      nostrServicesBuilder: buildProductionNostrServices,
      videoDeliveryBuilder: (settings, nostr) {
        return buildProductionVideoDelivery(
          settings,
          videoEnvironmentBuilder(nostr.ndk),
        );
      },
    );
  }

  final PreferencesLoader preferencesLoader;
  final ProductionNostrServicesBuilder nostrServicesBuilder;
  final ProductionVideoDeliveryBuilder videoDeliveryBuilder;
}

Future<AppDependencies> buildProductionDependencies([
  ProductionDependenciesEnvironment? environment,
]) async {
  WidgetsFlutterBinding.ensureInitialized();
  final bootstrap =
      environment ?? ProductionDependenciesEnvironment.production();
  final preferences = await bootstrap.preferencesLoader();
  final settingsRepository = LocalAppSettingsRepository(preferences);
  final settings = await settingsRepository.load();
  final nostr = bootstrap.nostrServicesBuilder(settings);
  final delivery = await bootstrap.videoDeliveryBuilder(settings, nostr);
  return composeProductionDependencies(
    preferences,
    settingsRepository,
    nostr,
    delivery,
  );
}

AppDependencies composeProductionDependencies(
  SharedPreferences preferences,
  LocalAppSettingsRepository settingsRepository,
  ProductionNostrServices nostr,
  ProductionVideoDelivery delivery,
) {
  return AppDependencies(
    sessionRepository: SecureSessionRepository(
      SecureSecretStore(const FlutterSecureStorage()),
      const NdkNostrIdentityDeriver(),
      nostr.adapters.session,
    ),
    appSettingsRepository: settingsRepository,
    videoCatalogServices: _buildVideoCatalog(preferences, delivery, nostr),
    activityRepository: NostrActivityRepository(
      client: nostr.eventClient,
      local: LocalActivityRepository(preferences),
    ),
    mediaPickerPort: ImagePickerMediaPicker(ImagePicker()),
    videoPlaybackPort: InventoryVideoPlaybackPort(
      delegate: const VideoPlayerPlaybackPort(),
      inventory: delivery.inventory,
    ),
    failureReporter: const DeveloperFailureReporter(),
  );
}

VideoCatalogServices _buildVideoCatalog(
  SharedPreferences preferences,
  ProductionVideoDelivery delivery,
  ProductionNostrServices nostr,
) {
  final local = LocalVideoStore(preferences);
  const reporter = DeveloperFailureReporter();
  final social = SocialGraphCache(nostr.adapters.social, local, reporter);
  final interactions = NostrVideoInteractions(
    NostrEngagementRepository(nostr.eventClient),
    NostrCommentsRepository(nostr.eventClient),
    reporter,
  );
  final reader = HybridVideoReader(
    remote: delivery.remoteSource,
    local: local,
    interactions: interactions,
    failureReporter: reporter,
  );
  return VideoCatalogServices(
    feed: FilteredVideoFeedRepository(reader, social),
    engagement: HybridVideoEngagementRepository(interactions),
    profile: AggregatingVideoProfileRepository(reader, social),
    search: FilteredVideoSearchRepository(reader, social),
    publishing: HybridVideoPublishingRepository(local, nostr.publisher),
    comments: HybridVideoCommentsRepository(interactions),
  );
}
