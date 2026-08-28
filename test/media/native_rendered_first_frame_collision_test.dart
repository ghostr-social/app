import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/domain/rendered_first_frame_port.dart';
import 'package:ghostr/platform/media/native_rendered_first_frame_port.dart';

void main() {
  test('a repeated token cannot alias an active frame attempt', () async {
    final events = StreamController<Object?>();
    final token = RenderedFirstFrameAttemptToken.parse(
      'abcdefghijklmnopqrstuA',
    );
    final port = NativeRenderedFirstFramePort(
      events: events.stream,
      tokenFactory: () => token,
    );

    final first = port.beginAttempt();
    final collision = port.beginAttempt();

    expect(first?.token, same(token));
    expect(collision, isNull);
    await port.dispose();
    await events.close();
  });
}
