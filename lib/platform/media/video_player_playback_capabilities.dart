import 'package:flutter/foundation.dart';
import 'package:ghostr/core/media/video_playback_capabilities.dart';

VideoPlaybackCapabilities currentVideoPlayerPlaybackCapabilities() {
  return videoPlayerPlaybackCapabilities(
    platform: defaultTargetPlatform,
    isWeb: kIsWeb,
  );
}

VideoPlaybackCapabilities videoPlayerPlaybackCapabilities({
  required TargetPlatform platform,
  required bool isWeb,
}) {
  if (isWeb || !_hasNativeVideoPlayerBackend(platform)) {
    return VideoPlaybackCapabilities.none;
  }
  return VideoPlaybackCapabilities.progressiveAndHls;
}

bool _hasNativeVideoPlayerBackend(TargetPlatform platform) {
  return switch (platform) {
    TargetPlatform.android ||
    TargetPlatform.iOS ||
    TargetPlatform.macOS =>
      true,
    TargetPlatform.fuchsia ||
    TargetPlatform.linux ||
    TargetPlatform.windows =>
      false,
  };
}
