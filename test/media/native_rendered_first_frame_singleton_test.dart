import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/native_rendered_first_frame_port.dart';

void main() {
  test('production listener is process-wide', () async {
    final events = StreamController<Object?>();

    final first = NativeRenderedFirstFramePort.production(
      events: events.stream,
    );
    final second = NativeRenderedFirstFramePort.production();

    expect(identical(first, second), isTrue);
    await first.dispose();
    await events.close();
  });
}
