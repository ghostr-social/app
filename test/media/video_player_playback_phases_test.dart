import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/recording_playback_telemetry_port.dart';
import '../support/scripted_video_player_platform.dart';

void main() {
  testWidgets(
    'reports startup, playback, real stall, pause, and end distinctly',
    (tester) async {
      final platform = ScriptedVideoPlayerPlatform();
      VideoPlayerPlatform.instance = platform;
      final telemetry = RecordingPlaybackTelemetryPort();
      final port = VideoPlayerPlaybackPort(telemetry: telemetry);

      await tester.pumpWidget(
        MaterialApp(
          home: port.buildSurface(
            media: VideoMediaSource.local('/cache/clip.mp4'),
            videoId: PlaybackVideoId.parse('clip'),
            isActive: true,
          ),
        ),
      );
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 100));

      expect(phases(telemetry).last, PlaybackPhase.starting);
      platform.emit(VideoEvent(eventType: VideoEventType.bufferingStart));
      platform.emit(
        VideoEvent(
          eventType: VideoEventType.isPlayingStateUpdate,
          isPlaying: false,
        ),
      );
      await tester.pump();
      expect(phases(telemetry).last, PlaybackPhase.starting);
      platform.emit(VideoEvent(eventType: VideoEventType.bufferingEnd));
      platform.emit(
        VideoEvent(
          eventType: VideoEventType.isPlayingStateUpdate,
          isPlaying: true,
        ),
      );
      await tester.pump();
      platform.position = const Duration(seconds: 1);
      await tester.pump(const Duration(milliseconds: 100));
      expect(phases(telemetry).last, PlaybackPhase.playing);
      platform.emit(
        VideoEvent(
          eventType: VideoEventType.bufferingUpdate,
          buffered: [DurationRange(Duration.zero, const Duration(seconds: 5))],
        ),
      );
      await tester.pump();
      expect(
        telemetry.observations.last.bufferedExtent,
        const Duration(seconds: 5),
      );
      platform.emit(VideoEvent(eventType: VideoEventType.bufferingStart));
      platform.emit(
        VideoEvent(
          eventType: VideoEventType.isPlayingStateUpdate,
          isPlaying: false,
        ),
      );
      await tester.pump();
      expect(phases(telemetry).last, PlaybackPhase.networkStalled);

      platform.emit(VideoEvent(eventType: VideoEventType.bufferingEnd));
      await tester.pump();
      expect(phases(telemetry).last, PlaybackPhase.paused);
      platform.emit(VideoEvent(eventType: VideoEventType.completed));
      await tester.pump();
      expect(phases(telemetry).last, PlaybackPhase.ended);
    },
  );
}

List<PlaybackPhase> phases(RecordingPlaybackTelemetryPort telemetry) =>
    telemetry.observations.map((item) => item.phase).toList();
