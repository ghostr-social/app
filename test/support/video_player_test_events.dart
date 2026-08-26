import 'package:flutter/services.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

final initializedVideoEvent = VideoEvent(
  eventType: VideoEventType.initialized,
  size: const Size(180, 320),
  duration: const Duration(seconds: 30),
);

PlatformException initializationError(String code) {
  return PlatformException(code: code, message: 'Source failed');
}
