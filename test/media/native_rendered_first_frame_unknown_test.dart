import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/domain/rendered_first_frame_port.dart';
import 'package:ghostr/platform/media/native_rendered_first_frame_port.dart';

void main() {
  test('drops native frames emitted before token issuance', () async {
    final events = StreamController<Object?>();
    final token = RenderedFirstFrameAttemptToken.parse(
      'abcdefghijklmnopqrstuA',
    );
    final port = NativeRenderedFirstFramePort(
      events: events.stream,
      tokenFactory: () => token,
    );
    events.add({'version': 1, 'attemptToken': token.value});
    await Future<void>.delayed(Duration.zero);
    var frames = 0;

    final attempt = port.beginAttempt()!;
    attempt.listen(() => frames += 1);
    expect(frames, 0);
    events.add({'version': 1, 'attemptToken': token.value});
    await Future<void>.delayed(Duration.zero);

    expect(frames, 1);
    await port.dispose();
    await events.close();
  });
}
