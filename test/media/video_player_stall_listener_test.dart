import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/video_player_value_listener.dart';
import 'package:video_player/video_player.dart';

void main() {
  test('flips the stalled state as playback buffering changes', () {
    final notifier = ValueNotifier<VideoPlayerValue>(
      const VideoPlayerValue(duration: Duration.zero),
    );
    var changes = 0;
    final listener = VideoPlayerValueListener(
      onStateChanged: () => changes += 1,
    );

    listener.attach(notifier);
    expect(listener.isStalled, isFalse);

    notifier.value = const VideoPlayerValue(
      duration: Duration(seconds: 10),
      isInitialized: true,
      isBuffering: true,
    );
    expect(listener.isStalled, isTrue);
    expect(changes, 1);

    notifier.value = const VideoPlayerValue(
      duration: Duration(seconds: 10),
      isInitialized: true,
    );
    expect(listener.isStalled, isFalse);
    expect(changes, 2);

    listener.detach();
    notifier.value = const VideoPlayerValue(
      duration: Duration(seconds: 10),
      isInitialized: true,
      isBuffering: true,
    );
    expect(listener.isStalled, isFalse);
    expect(changes, 2);
  });
}
