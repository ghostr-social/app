import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/platform/media/native_rendered_first_frame_port.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

import 'device_player_preparation_feedback.dart';
import 'device_playback_authority.dart';
import 'device_playback_probe.dart';
import 'device_video_scenario.dart';
import 'device_video_server.dart';

export 'device_playback_probe.dart';

part 'device_playback_testbed_wait.dart';

final class DevicePlaybackTestbed {
  DevicePlaybackTestbed._(this.server, this.probe, this._firstFrames)
    : _playback = VideoPlayerPlaybackPort(
        telemetry: probe,
        preparationFeedback: DevicePlayerPreparationFeedback(),
        renderedFirstFrames: _firstFrames,
      );

  static Future<DevicePlaybackTestbed> start(
    DeviceVideoScenario scenario,
  ) async {
    final server = await DeviceVideoServer.start(scenario);
    return DevicePlaybackTestbed._(
      server,
      DevicePlaybackProbe(),
      NativeRenderedFirstFramePort(),
    );
  }

  final DeviceVideoServer server;
  final DevicePlaybackProbe probe;
  final NativeRenderedFirstFramePort _firstFrames;
  final VideoPlayerPlaybackPort _playback;
  bool _closed = false;
  bool _shellMounted = false;

  Future<PlaybackFocus> show(
    WidgetTester tester,
    String rawVideoId, {
    bool isActive = true,
  }) async {
    await _ensureShellMounted(tester);
    final videoId = PlaybackVideoId.parse(rawVideoId);
    final media = ProxiedHlsVideoMediaSource(
      server.playbackUri(rawVideoId).toString(),
    );
    final authority = devicePlaybackFixtureAuthority(media);
    final focus = probe.markFocus(videoId);
    await tester.pumpWidget(
      MaterialApp(
        home: SizedBox.expand(
          child: _playback.buildSurface(
            VideoPlaybackSurfaceRequest(
              media: media,
              videoId: videoId,
              isActive: isActive,
              authority: authority,
            ),
          ),
        ),
      ),
    );
    return focus;
  }

  Future<void> _ensureShellMounted(WidgetTester tester) async {
    if (_shellMounted) return;
    await tester.pumpWidget(const MaterialApp(home: SizedBox.expand()));
    _shellMounted = true;
  }

  Future<void> close() async {
    if (_closed) return;
    _closed = true;
    try {
      await _firstFrames.dispose();
    } finally {
      await server.close();
    }
  }
}
