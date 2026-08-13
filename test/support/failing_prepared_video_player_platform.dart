import 'package:flutter/services.dart';

import 'fake_video_player_platform.dart';

final class PauseFailingPreparedVideoPlayerPlatform
    extends FakeVideoPlayerPlatform {
  var _pauseCalls = 0;

  @override
  Future<void> pause(int textureId) async {
    _pauseCalls += 1;
    if (_pauseCalls == 2) {
      throw PlatformException(code: 'pause-failed');
    }
    await super.pause(textureId);
  }
}

final class SeekFailingPreparedVideoPlayerPlatform
    extends FakeVideoPlayerPlatform {
  @override
  Future<void> seekTo(int textureId, Duration position) {
    throw PlatformException(code: 'seek-failed');
  }
}
