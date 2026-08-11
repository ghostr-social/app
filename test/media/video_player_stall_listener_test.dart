import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/video_player_value_listener.dart';
import 'package:video_player/video_player.dart';

void main() {
  test('forwards player values only while attached', () {
    final notifier = ValueNotifier<VideoPlayerValue>(
      const VideoPlayerValue(duration: Duration.zero),
    );
    final values = <VideoPlayerValue>[];
    final listener = VideoPlayerValueListener(onValueChanged: values.add);

    listener.attach(notifier);
    expect(values, hasLength(1));

    notifier.value = const VideoPlayerValue(
      duration: Duration(seconds: 10),
      isInitialized: true,
      isBuffering: true,
    );
    expect(values.last.isBuffering, isTrue);

    listener.detach();
    notifier.value = const VideoPlayerValue(
      duration: Duration(seconds: 10),
      isInitialized: true,
      isBuffering: true,
    );
    expect(values, hasLength(2));
  });
}
