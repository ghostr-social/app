import 'package:flutter/widgets.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:ghostr/app/app_dependencies.dart';
import 'package:ghostr/app/production_nostr_services.dart';
import 'package:ghostr/app/production_video_catalog.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/app/production_video_playback.dart';
import 'package:ghostr/app/production_video_sharing.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_client.dart';
import 'package:ghostr/core/storage/account_storage_scope.dart';
import 'package:ghostr/features/activity/data/local_activity_repository.dart';
import 'package:ghostr/features/activity/data/nostr_activity_repository.dart';
import 'package:ghostr/features/session/data/ndk_nostr_identity_deriver.dart';
import 'package:ghostr/features/session/data/secure_session_repository.dart';
import 'package:ghostr/features/settings/data/local_app_settings_repository.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';
import 'package:ghostr/features/watch_history/data/local_watch_history_repository.dart';
import 'package:ghostr/platform/logging/developer_failure_reporter.dart';
import 'package:ghostr/platform/media/image_picker_capabilities.dart';
import 'package:ghostr/platform/media/image_picker_media_picker.dart';
import 'package:ghostr/platform/storage/secure_secret_store.dart';
import 'package:image_picker/image_picker.dart';
import 'package:shared_preferences/shared_preferences.dart';

export 'production_nostr_services.dart';

typedef PreferencesLoader = Future<SharedPreferences> Function();
typedef ProductionNostrServicesBuilder =
    ProductionNostrServices Function(AppSettings settings);
typedef ProductionVideoDeliveryBuilder =
    Future<ProductionVideoDelivery> Function(
      AppSettings settings,
      ProductionNostrServices nostr,
    );
typedef ProductionVideoEnvironmentBuilder =
    ProductionVideoDeliveryEnvironment Function(RustFeedViewer viewer);

/// Reads the signed-in account on demand. A missing account is a null viewer.
RustFeedViewer signedInViewer(NostrEventClient client) {
  return () {
    try {
      return client.publicKeyHex;
    } on AppFailure {
      return null;
    }
  };
}

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
          videoEnvironmentBuilder(signedInViewer(nostr.eventClient)),
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
  final accountScope = AccountStorageScope(
    () => nostr.eventClient.publicKeyHex,
  );
  final watchHistory = LocalWatchHistoryRepository(
    preferences,
    accountScope: accountScope,
  );
  return AppDependencies(
    sessionRepository: SecureSessionRepository(
      SecureSecretStore(const FlutterSecureStorage()),
      const NdkNostrIdentityDeriver(),
      nostr.adapters.session,
    ),
    appSettingsRepository: settingsRepository,
    videoCatalogServices: buildProductionVideoCatalog(
      ProductionVideoCatalogInputs(
        preferences: preferences,
        delivery: delivery,
        nostr: nostr,
        accountScope: accountScope,
        watchHistory: watchHistory,
        settingsRepository: settingsRepository,
      ),
    ),
    watchHistoryRepository: watchHistory,
    activityRepository: NostrActivityRepository(
      client: nostr.eventClient,
      local: LocalActivityRepository(preferences, accountScope: accountScope),
      failureReporter: const DeveloperFailureReporter(),
    ),
    mediaPickerPort: ImagePickerMediaPicker(
      ImagePicker(),
      capabilities: currentImagePickerCapabilities(),
    ),
    videoPlaybackPort: buildProductionVideoPlayback(delivery),
    videoShareWorkflow: buildProductionVideoSharing(),
    failureReporter: const DeveloperFailureReporter(),
  );
}
