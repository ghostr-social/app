import 'package:ghostr/app/app_dependencies.dart';
import 'package:ghostr/app/video_catalog_services.dart';
import 'package:ghostr/app/video_feed_binding.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:ghostr/features/session/data/pending_first_session_repository.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'fake_activity_repository.dart';
import 'fake_device_dependencies.dart';
import 'fake_dependency_overrides.dart';
import 'default_account_generator.dart';
import 'fake_app_settings_repository.dart';
import 'fake_incoming_video_share_port.dart';
import 'fake_media_ports.dart';
import 'fake_session_repository.dart';
import 'fake_video_catalog_repository.dart';
import 'fake_video_sharing.dart';
import 'fake_watch_history_repository.dart';
import 'recording_failure_reporter.dart';
import 'fake_profile_metadata_repository.dart';
import 'fake_profile_image_services.dart';
import 'fake_secret_backup_port.dart';
import 'fake_account_provisioning_repository.dart';

export 'fake_device_dependencies.dart';
export 'fake_dependency_overrides.dart';

AppDependencies buildFakeDependencies({
  UserSession? session,
  required FakeVideoCatalogRepository catalogRepository,
  FakeDeviceDependencies device = const FakeDeviceDependencies(),
  FakeDependencyOverrides overrides = const FakeDependencyOverrides(),
}) {
  final provisioning =
      overrides.accountProvisioningRepository ??
      FakeAccountProvisioningRepository();
  final sessions =
      overrides.sessionRepository ??
      FakeSessionRepository(storedSession: session);
  return AppDependencies(
    sessionRepository: PendingFirstSessionRepository(sessions, provisioning),
    appSettingsRepository: FakeAppSettingsRepository(AppSettings.defaults()),
    videoCatalogServices: VideoCatalogServices(
      feed: VideoFeedBinding(
        repository: overrides.feed ?? catalogRepository,
        updates: overrides.feedUpdates,
      ),
      discovery: VideoCatalogDiscoveryServices(
        profile: catalogRepository,
        search: catalogRepository,
        searchUpdates: catalogRepository,
        trending: catalogRepository,
      ),
      interactions: VideoCatalogInteractionServices(
        engagement: catalogRepository,
        comments: catalogRepository,
        social: catalogRepository,
        reposts: catalogRepository,
      ),
      authoring: VideoCatalogAuthoringServices(
        publishing: overrides.publishing ?? catalogRepository,
      ),
    ),
    activityRepository: device.activity ?? FakeActivityRepository(),
    accountGenerator: overrides.accountGenerator ?? fakeAccountGenerator(),
    accountProvisioningRepository: provisioning,
    profileMetadataRepository:
        overrides.profileMetadataRepository ?? FakeProfileMetadataRepository(),
    profileImageWorkflow: device.profileImages ?? fakeProfileImages(),
    secretBackupPort: device.secretBackup ?? FakeSecretBackupPort(),
    watchHistoryRepository:
        overrides.watchHistory ?? FakeWatchHistoryRepository(),
    incomingVideoSharePort:
        device.incomingVideoShares ?? FakeIncomingVideoSharePort(),
    mediaPickerPort: device.mediaPicker ?? FakeMediaPickerPort(),
    videoPlaybackPort: device.playback ?? FakeVideoPlaybackPort(),
    videoShareWorkflow: device.sharing ?? FakeVideoShareWorkflow(),
    failureReporter: RecordingFailureReporter(),
    appUpdateRuntime: overrides.appUpdateRuntime,
  );
}
