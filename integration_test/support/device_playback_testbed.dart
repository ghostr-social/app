import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

import 'device_playback_probe.dart';
import 'device_video_scenario.dart';
import 'device_video_server.dart';

part 'device_playback_testbed_wait.dart';

final class DevicePlaybackTestbed {
  DevicePlaybackTestbed._(this.server, this.probe);

  static Future<DevicePlaybackTestbed> start(
    DeviceVideoScenario scenario,
  ) async {
    final server = await DeviceVideoServer.start(scenario);
    return DevicePlaybackTestbed._(server, DevicePlaybackProbe());
  }

  final DeviceVideoServer server;
  final DevicePlaybackProbe probe;
  bool _semanticsEnabled = false;

  Future<PlaybackFocus> show(WidgetTester tester, String rawVideoId) async {
    _enableSemantics(tester);
    final videoId = PlaybackVideoId.parse(rawVideoId);
    final focus = probe.markFocus(videoId);
    final media = ProxiedHlsVideoMediaSource(
      server.playbackUri(rawVideoId).toString(),
    );
    await tester.pumpWidget(
      MaterialApp(
        home: SizedBox.expand(
          child: VideoPlayerPlaybackPort(telemetry: probe).buildSurface(
            VideoPlaybackSurfaceRequest(
              media: media,
              videoId: videoId,
              isActive: true,
            ),
          ),
        ),
      ),
    );
    return focus;
  }

  Future<void> close() => server.close();

  void _enableSemantics(WidgetTester tester) {
    if (_semanticsEnabled) return;
    _semanticsEnabled = true;
    final handle = tester.ensureSemantics();
    addTearDown(handle.dispose);
  }
}
