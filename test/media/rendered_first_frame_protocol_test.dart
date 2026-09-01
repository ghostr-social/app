import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/rendered_first_frame_protocol.dart';

void main() {
  test('native first-frame correlation uses the stable attempt header', () {
    expect(warpPlaybackAttemptHeader, 'X-Ghostr-Playback-Attempt');
  });
}
