import 'package:ghostr/core/media/media_picker_port.dart';
import 'package:ghostr/core/errors/failure_reporter.dart';
import 'package:ghostr/app/video_catalog_services.dart';
import 'package:ghostr/features/activity/domain/activity_repository.dart';
import 'package:ghostr/features/session/domain/session_repository.dart';
import 'package:ghostr/features/settings/domain/app_settings_repository.dart';
import 'package:ghostr/features/video_inventory/domain/video_inventory_port.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_repository.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

class AppDependencies {
  const AppDependencies({
    required this.sessionRepository,
    required this.appSettingsRepository,
    required this.videoCatalogServices,
    required this.activityRepository,
    required this.watchHistoryRepository,
    required this.mediaPickerPort,
    required this.videoPlaybackPort,
    required this.videoInventory,
    required this.failureReporter,
  });

  final SessionRepository sessionRepository;
  final AppSettingsRepository appSettingsRepository;
  final VideoCatalogServices videoCatalogServices;
  final ActivityRepository activityRepository;
  final WatchHistoryRepository watchHistoryRepository;
  final MediaPickerPort mediaPickerPort;
  final VideoPlaybackPort videoPlaybackPort;
  final VideoInventoryPort videoInventory;
  final FailureReporter failureReporter;
}
