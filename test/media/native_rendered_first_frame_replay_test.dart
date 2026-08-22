import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/domain/player_preparation_feedback_port.dart';
import 'package:ghostr/platform/media/native_rendered_first_frame_port.dart';

void main() {
  test('replays one early native frame and rejects duplicates', () async {
    final events = StreamController<Object?>();
    final port = NativeRenderedFirstFramePort(events: events.stream);
    final token = PlayerPreparationAttemptToken.parse('abcdefghijklmnopqrstuA');
    events.add({'version': 1, 'attemptToken': token.value});
    await Future<void>.delayed(Duration.zero);
    var frames = 0;

    port.register(token, () => frames += 1);
    events.add({'version': 1, 'attemptToken': token.value});
    await Future<void>.delayed(Duration.zero);

    expect(frames, 1);
    await port.dispose();
    await events.close();
  });
}
