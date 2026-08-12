import 'package:ghostr/app/app_dependencies.dart';
import 'package:ghostr/app/video_catalog_services.dart';
import 'package:ghostr/app/production_app_update.dart';
import 'package:ghostr/app/video_feed_binding.dart';
import 'package:ghostr/features/profile/domain/profile_image_workflow.dart';
import 'package:ghostr/features/profile/domain/profile_metadata_repository.dart';
import 'package:ghostr/features/session/domain/account_provisioning_repository.dart';
import 'package:ghostr/features/session/domain/session_repository.dart';
import 'package:ghostr/features/session/domain/nostr_account_generator.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:ghostr/features/session/data/pending_first_session_repository.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/publish/domain/video_publishing_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_updates.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_workflow.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'fake_activity_repository.dart';
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

AppDependencies buildFakeDependencies({
  UserSession? session,
  SessionRepository? sessionRepository,
  required FakeVideoCatalogRepository catalogRepository,
  VideoFeedUpdates? feedUpdates,
  FakeWatchHistoryRepository? watchHistory,
  VideoPublishingRepository? publishing,
  AppUpdateRuntime? appUpdateRuntime,
  FakeDeviceDependencies device = const FakeDeviceDependencies(),
  NostrAccountGenerator? accountGenerator,
  AccountProvisioningRepository? accountProvisioningRepository,
  ProfileMetadataRepository? profileMetadataRepository,
}) {
  final provisioning =
      accountProvisioningRepository ?? FakeAccountProvisioningRepository();
  final sessions =
      sessionRepository ?? FakeSessionRepository(storedSession: session);
  return AppDependencies(
    sessionRepository: PendingFirstSessionRepository(sessions, provisioning),
    appSettingsRepository: FakeAppSettingsRepository(AppSettings.defaults()),
    videoCatalogServices: VideoCatalogServices(
      feed: VideoFeedBinding(
        repository: catalogRepository,
        updates: feedUpdates,
      ),
      engagement: catalogRepository,
      profile: catalogRepository,
      search: catalogRepository,
      searchUpdates: catalogRepository,
      trending: catalogRepository,
      publishing: publishing ?? catalogRepository,
      comments: catalogRepository,
      social: catalogRepository,
    ),
    activityRepository: device.activity ?? FakeActivityRepository(),
    accountGenerator: accountGenerator ?? fakeAccountGenerator(),
    accountProvisioningRepository: provisioning,
    profileMetadataRepository:
        profileMetadataRepository ?? FakeProfileMetadataRepository(),
    profileImageWorkflow: device.profileImages ?? fakeProfileImages(),
    secretBackupPort: device.secretBackup ?? FakeSecretBackupPort(),
    watchHistoryRepository: watchHistory ?? FakeWatchHistoryRepository(),
    incomingVideoSharePort:
        device.incomingVideoShares ?? FakeIncomingVideoSharePort(),
    mediaPickerPort: device.mediaPicker ?? FakeMediaPickerPort(),
    videoPlaybackPort: device.playback ?? FakeVideoPlaybackPort(),
    videoShareWorkflow: device.sharing ?? FakeVideoShareWorkflow(),
    failureReporter: RecordingFailureReporter(),
    appUpdateRuntime: appUpdateRuntime,
  );
}

class FakeDeviceDependencies {
  const FakeDeviceDependencies({
    this.activity,
    this.incomingVideoShares,
    this.mediaPicker,
    this.playback,
    this.secretBackup,
    this.profileImages,
    this.sharing,
  });

  final FakeActivityRepository? activity;
  final FakeIncomingVideoSharePort? incomingVideoShares;
  final FakeMediaPickerPort? mediaPicker;
  final VideoPlaybackPort? playback;
  final FakeSecretBackupPort? secretBackup;
  final ProfileImageWorkflow? profileImages;
  final VideoShareWorkflow? sharing;
}
