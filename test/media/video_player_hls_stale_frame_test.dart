import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/native_rendered_first_frame_port.dart';
import 'package:ghostr/platform/media/rendered_first_frame_protocol.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/fake_video_player_platform.dart';
import '../support/recording_playback_telemetry_port.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('late HLS frame cannot present a replacement controller', (
    tester,
  ) async {
    final platform = FakeVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final events = StreamController<Object?>();
    final frames = NativeRenderedFirstFramePort(events: events.stream);
    final telemetry = RecordingPlaybackTelemetryPort();
    final port = VideoPlayerPlaybackPort(
      telemetry: telemetry,
      renderedFirstFrames: frames,
    );
    addTearDown(() async {
      await frames.dispose();
      await events.close();
    });

    await pumpVideoPlayerSurface(tester, port, _request('a', 'post-a'));
    await settleVideoPlayerTasks(tester);
    final stale = _token(platform, 0);
    await pumpVideoPlayerSurface(tester, port, _request('b', 'post-b'));
    await settleVideoPlayerTasks(tester);
    final current = _token(platform, 1);

    events.add({'version': 1, 'attemptToken': stale});
    await settleVideoPlayerTasks(tester);
    expect(telemetry.presentations, isEmpty);
    events.add({'version': 1, 'attemptToken': current});
    await settleVideoPlayerTasks(tester);

    expect(telemetry.presentations.single.deliveryId.value, 'post-b');
  });
}

VideoPlaybackSurfaceRequest _request(String session, String delivery) {
  final id = session * 64;
  return VideoPlaybackSurfaceRequest(
    media: ProxiedHlsVideoMediaSource(
      'http://127.0.0.1:3210/hls/$id/index.m3u8',
    ),
    videoId: PlaybackVideoId.parse('clip'),
    isActive: true,
    playbackDeliveryId: PlaybackDeliveryId.parse(delivery),
  );
}

String _token(FakeVideoPlayerPlatform platform, int index) {
  return platform.dataSources[index].httpHeaders[warpPlaybackAttemptHeader]!;
}
