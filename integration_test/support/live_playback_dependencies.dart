import 'package:ghostr/app/app_dependencies.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

// Copy composition only: all account, storage and network services stay real.
AppDependencies livePlaybackDependencies(
  AppDependencies production,
  VideoPlaybackPort playback,
) => AppDependencies(
  sessionRepository: production.sessionRepository,
  appSettingsRepository: production.appSettingsRepository,
  videoCatalogServices: production.videoCatalogServices,
  activityRepository: production.activityRepository,
  accountGenerator: production.accountGenerator,
  accountProvisioningRepository: production.accountProvisioningRepository,
  profileMetadataRepository: production.profileMetadataRepository,
  profileImageWorkflow: production.profileImageWorkflow,
  secretBackupPort: production.secretBackupPort,
  watchHistoryRepository: production.watchHistoryRepository,
  incomingVideoSharePort: production.incomingVideoSharePort,
  mediaPickerPort: production.mediaPickerPort,
  videoPlaybackPort: playback,
  videoShareWorkflow: production.videoShareWorkflow,
  failureReporter: production.failureReporter,
  playbackPreparationUpdates: production.playbackPreparationUpdates,
  appUpdateRuntime: production.appUpdateRuntime,
  watchHistoryStorageDisposer: production.watchHistoryStorageDisposer,
);
