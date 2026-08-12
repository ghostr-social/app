import 'package:ghostr/app/app_dependencies.dart';
import 'package:ghostr/app/video_catalog_services.dart';
import 'package:ghostr/app/video_feed_binding.dart';
import 'package:ghostr/features/session/domain/session_repository.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/publish/domain/video_publishing_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_updates.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_workflow.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

import 'fake_activity_repository.dart';
import 'fake_app_settings_repository.dart';
import 'fake_incoming_video_share_port.dart';
import 'fake_media_ports.dart';
import 'fake_session_repository.dart';
import 'fake_video_catalog_repository.dart';
import 'fake_video_sharing.dart';
import 'fake_watch_history_repository.dart';
import 'recording_failure_reporter.dart';

AppDependencies buildFakeDependencies({
  UserSession? session,
  SessionRepository? sessionRepository,
  required FakeVideoCatalogRepository catalogRepository,
  VideoFeedUpdates? feedUpdates,
  FakeWatchHistoryRepository? watchHistory,
  VideoPublishingRepository? publishing,
  FakeDeviceDependencies device = const FakeDeviceDependencies(),
}) {
  return AppDependencies(
    sessionRepository:
        sessionRepository ?? FakeSessionRepository(storedSession: session),
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
    watchHistoryRepository: watchHistory ?? FakeWatchHistoryRepository(),
    incomingVideoSharePort:
        device.incomingVideoShares ?? FakeIncomingVideoSharePort(),
    mediaPickerPort: device.mediaPicker ?? FakeMediaPickerPort(),
    videoPlaybackPort: device.playback ?? FakeVideoPlaybackPort(),
    videoShareWorkflow: device.sharing ?? FakeVideoShareWorkflow(),
    failureReporter: RecordingFailureReporter(),
  );
}

class FakeDeviceDependencies {
  const FakeDeviceDependencies({
    this.activity,
    this.incomingVideoShares,
    this.mediaPicker,
    this.playback,
    this.sharing,
  });

  final FakeActivityRepository? activity;
  final FakeIncomingVideoSharePort? incomingVideoShares;
  final FakeMediaPickerPort? mediaPicker;
  final VideoPlaybackPort? playback;
  final VideoShareWorkflow? sharing;
}
