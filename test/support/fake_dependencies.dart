import 'package:ghostr/app/app_dependencies.dart';
import 'package:ghostr/app/video_catalog_services.dart';
import 'package:ghostr/features/session/domain/session_repository.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

import 'fake_activity_repository.dart';
import 'fake_app_settings_repository.dart';
import 'fake_media_ports.dart';
import 'fake_session_repository.dart';
import 'fake_video_catalog_repository.dart';
import 'fake_watch_history_repository.dart';
import 'recording_failure_reporter.dart';

AppDependencies buildFakeDependencies({
  UserSession? session,
  SessionRepository? sessionRepository,
  required FakeVideoCatalogRepository catalogRepository,
  FakeWatchHistoryRepository? watchHistory,
  FakeDeviceDependencies device = const FakeDeviceDependencies(),
}) {
  return AppDependencies(
    sessionRepository:
        sessionRepository ?? FakeSessionRepository(storedSession: session),
    appSettingsRepository: FakeAppSettingsRepository(AppSettings.defaults()),
    videoCatalogServices: VideoCatalogServices(
      feed: catalogRepository,
      engagement: catalogRepository,
      profile: catalogRepository,
      search: catalogRepository,
      trending: catalogRepository,
      publishing: catalogRepository,
      comments: catalogRepository,
      social: catalogRepository,
    ),
    activityRepository: device.activity ?? FakeActivityRepository(),
    watchHistoryRepository: watchHistory ?? FakeWatchHistoryRepository(),
    mediaPickerPort: device.mediaPicker ?? FakeMediaPickerPort(),
    videoPlaybackPort: device.playback ?? FakeVideoPlaybackPort(),
    failureReporter: RecordingFailureReporter(),
  );
}

class FakeDeviceDependencies {
  const FakeDeviceDependencies({
    this.activity,
    this.mediaPicker,
    this.playback,
  });

  final FakeActivityRepository? activity;
  final FakeMediaPickerPort? mediaPicker;
  final VideoPlaybackPort? playback;
}
