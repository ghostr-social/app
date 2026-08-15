import 'package:ghostr/core/media/media_picker_port.dart';
import 'package:ghostr/core/media/incoming_video_share.dart';
import 'package:ghostr/core/errors/failure_reporter.dart';
import 'package:ghostr/app/video_catalog_services.dart';
import 'package:ghostr/app/production_app_update.dart';
import 'package:ghostr/features/activity/domain/activity_repository.dart';
import 'package:ghostr/features/profile/domain/profile_metadata_repository.dart';
import 'package:ghostr/features/profile/domain/profile_image_workflow.dart';
import 'package:ghostr/features/session/domain/nostr_account_generator.dart';
import 'package:ghostr/features/session/domain/account_provisioning_repository.dart';
import 'package:ghostr/features/session/domain/session_repository.dart';
import 'package:ghostr/features/session/domain/secret_backup_port.dart';
import 'package:ghostr/features/settings/domain/app_settings_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_repository.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_workflow.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

typedef AsyncDisposer = Future<void> Function();

class AppDependencies {
  const AppDependencies({
    required this.sessionRepository,
    required this.appSettingsRepository,
    required this.videoCatalogServices,
    required this.activityRepository,
    required this.accountGenerator,
    required this.accountProvisioningRepository,
    required this.profileMetadataRepository,
    required this.profileImageWorkflow,
    required this.secretBackupPort,
    required this.watchHistoryRepository,
    required this.incomingVideoSharePort,
    required this.mediaPickerPort,
    required this.videoPlaybackPort,
    required this.videoShareWorkflow,
    required this.failureReporter,
    this.appUpdateRuntime,
    this.watchHistoryStorageDisposer,
  });

  final SessionRepository sessionRepository;
  final AppSettingsRepository appSettingsRepository;
  final VideoCatalogServices videoCatalogServices;
  final ActivityRepository activityRepository;
  final NostrAccountGenerator accountGenerator;
  final AccountProvisioningRepository accountProvisioningRepository;
  final ProfileMetadataRepository profileMetadataRepository;
  final ProfileImageWorkflow profileImageWorkflow;
  final SecretBackupPort secretBackupPort;
  final WatchHistoryRepository watchHistoryRepository;
  final IncomingVideoSharePort incomingVideoSharePort;
  final MediaPickerPort mediaPickerPort;
  final VideoPlaybackPort videoPlaybackPort;
  final VideoShareWorkflow videoShareWorkflow;
  final FailureReporter failureReporter;
  final AppUpdateRuntime? appUpdateRuntime;
  final AsyncDisposer? watchHistoryStorageDisposer;

  Future<void> close() {
    final updateRuntime = appUpdateRuntime;
    final historyDisposer = watchHistoryStorageDisposer;
    return Future.wait([
      Future<void>.sync(incomingVideoSharePort.close),
      if (updateRuntime != null) Future<void>.sync(updateRuntime.dispose),
      if (historyDisposer != null) Future<void>.sync(historyDisposer),
    ]);
  }
}
