import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/recording_playback_telemetry_port.dart';
import '../support/scripted_video_player_platform.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets(
    'reports startup, playback, real stall, pause, and end distinctly',
    (tester) async {
      final platform = ScriptedVideoPlayerPlatform();
      VideoPlayerPlatform.instance = platform;
      final telemetry = RecordingPlaybackTelemetryPort();
      final port = VideoPlayerPlaybackPort(telemetry: telemetry);
      await pumpVideoPlayerSurface(
        tester,
        port,
        VideoPlaybackSurfaceRequest(
          media: VideoMediaSource.local('/cache/clip.mp4'),
          videoId: PlaybackVideoId.parse('clip'),
          isActive: true,
        ),
      );
      await _expectStartup(tester, platform, telemetry);
      await _expectPlayback(tester, platform, telemetry);
      await _expectStallAndTerminalPhases(tester, platform, telemetry);
    },
  );
}

Future<void> _expectStartup(
  WidgetTester tester,
  ScriptedVideoPlayerPlatform platform,
  RecordingPlaybackTelemetryPort telemetry,
) async {
  expect(phases(telemetry).last, PlaybackPhase.starting);
  platform.emit(VideoEvent(eventType: VideoEventType.bufferingStart));
  _emitPlaying(platform, false);
  await tester.pump();
  expect(phases(telemetry).last, PlaybackPhase.starting);
}

Future<void> _expectPlayback(
  WidgetTester tester,
  ScriptedVideoPlayerPlatform platform,
  RecordingPlaybackTelemetryPort telemetry,
) async {
  platform.emit(VideoEvent(eventType: VideoEventType.bufferingEnd));
  _emitPlaying(platform, true);
  await tester.pump();
  platform.position = const Duration(seconds: 1);
  await tester.pump(const Duration(milliseconds: 100));
  expect(phases(telemetry).last, PlaybackPhase.playing);
  platform.emit(_bufferedFiveSeconds);
  await tester.pump();
  expect(telemetry.observations.last.bufferedExtent, _fiveSeconds);
}

Future<void> _expectStallAndTerminalPhases(
  WidgetTester tester,
  ScriptedVideoPlayerPlatform platform,
  RecordingPlaybackTelemetryPort telemetry,
) async {
  platform.emit(VideoEvent(eventType: VideoEventType.bufferingStart));
  _emitPlaying(platform, false);
  await tester.pump();
  expect(phases(telemetry).last, PlaybackPhase.networkStalled);
  platform.emit(VideoEvent(eventType: VideoEventType.bufferingEnd));
  await tester.pump();
  expect(phases(telemetry).last, PlaybackPhase.paused);
  platform.emit(VideoEvent(eventType: VideoEventType.completed));
  await tester.pump();
  expect(phases(telemetry).last, PlaybackPhase.ended);
}

void _emitPlaying(ScriptedVideoPlayerPlatform platform, bool isPlaying) {
  platform.emit(
    VideoEvent(
      eventType: VideoEventType.isPlayingStateUpdate,
      isPlaying: isPlaying,
    ),
  );
}

const _fiveSeconds = Duration(seconds: 5);
final _bufferedFiveSeconds = VideoEvent(
  eventType: VideoEventType.bufferingUpdate,
  buffered: [DurationRange(Duration.zero, _fiveSeconds)],
);

List<PlaybackPhase> phases(RecordingPlaybackTelemetryPort telemetry) =>
    telemetry.observations.map((item) => item.phase).toList();
