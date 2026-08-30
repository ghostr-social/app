import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/fake_video_player_platform.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('retention changes cannot queue a rewind before reactivation', (
    tester,
  ) async {
    final platform = _BlockingSeekPlatform();
    final pause = platform.pauseBarrier = Completer<void>();
    final media = VideoMediaSource.local('/cache/warm.mp4');
    final port = VideoPlayerPlaybackPort();
    VideoPlayerPlatform.instance = platform;
    addTearDown(() {
      if (!pause.isCompleted) pause.complete();
      platform.releaseSeek();
    });

    await _show(tester, port, media, active: true);
    await settleVideoPlayerTasks(tester);
    expect(platform.calls.where((call) => call == 'play'), hasLength(1));
    await _show(tester, port, media, active: false, keepWarm: true);
    expect(platform.calls, contains('pause'));
    await _show(tester, port, media, active: false);

    pause.complete();
    await settleVideoPlayerTasks(tester);
    await _show(tester, port, media, active: true);
    await settleVideoPlayerTasks(tester);

    expect(
      platform.calls.where((call) => call == 'play'),
      hasLength(2),
      reason: '${platform.calls}',
    );
  });
}

Future<void> _show(
  WidgetTester tester,
  VideoPlayerPlaybackPort port,
  VideoMediaSource media, {
  required bool active,
  bool keepWarm = false,
}) {
  return pumpVideoPlayerSurface(
    tester,
    port,
    VideoPlaybackSurfaceRequest(
      media: media,
      isActive: active,
      keepWarmWhenInactive: keepWarm,
    ),
  );
}

final class _BlockingSeekPlatform extends FakeVideoPlayerPlatform {
  final _seekRelease = Completer<void>();

  @override
  Future<void> seekTo(int textureId, Duration position) async {
    await super.seekTo(textureId, position);
    await _seekRelease.future;
  }

  void releaseSeek() {
    if (!_seekRelease.isCompleted) _seekRelease.complete();
  }
}
