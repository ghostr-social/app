import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/domain/player_preparation_feedback_port.dart';
import 'package:ghostr/platform/media/native_rendered_first_frame_port.dart';

void main() {
  test('retains only the eight newest unmatched native frames', () async {
    final events = StreamController<Object?>();
    final port = NativeRenderedFirstFramePort(events: events.stream);
    final tokens = List.generate(9, _token);
    for (final token in tokens) {
      events.add({'version': 1, 'attemptToken': token.value});
    }
    await Future<void>.delayed(Duration.zero);
    var oldestFrames = 0;
    var newestFrames = 0;

    port.register(tokens.first, () => oldestFrames += 1);
    port.register(tokens.last, () => newestFrames += 1);

    expect(oldestFrames, 0);
    expect(newestFrames, 1);
    await port.dispose();
    await events.close();
  });
}

PlayerPreparationAttemptToken _token(int index) {
  return PlayerPreparationAttemptToken.parse(
    '${index.toRadixString(16).padLeft(21, 'a')}A',
  );
}
