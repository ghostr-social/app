import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/domain/player_preparation_feedback_port.dart';
import 'package:ghostr/platform/media/native_rendered_first_frame_port.dart';

void main() {
  test('accepts only the exact versioned native event schema', () async {
    final events = StreamController<Object?>();
    final port = NativeRenderedFirstFramePort(events: events.stream);
    final token = PlayerPreparationAttemptToken.parse('abcdefghijklmnopqrstuA');
    var frames = 0;
    port.register(token, () => frames += 1);

    events.add({'version': 2, 'attemptToken': token.value});
    events.add({'version': 1, 'attemptToken': '${token.value}x'});
    events.add({'version': 1, 'attemptToken': token.value, 'extra': true});
    await Future<void>.delayed(Duration.zero);
    expect(frames, 0);

    events.add({'version': 1, 'attemptToken': token.value});
    await Future<void>.delayed(Duration.zero);
    expect(frames, 1);
    await port.dispose();
    await events.close();
  });
}
