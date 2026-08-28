import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/domain/rendered_first_frame_port.dart';

void main() {
  test('native attempt token is exactly 128-bit base64url', () {
    const valid = 'abcdefghijklmnopqrstuA';

    expect(RenderedFirstFrameAttemptToken.parse(valid).value, valid);
    expect(
      () => RenderedFirstFrameAttemptToken.parse('too-short'),
      throwsFormatException,
    );
    expect(
      () => RenderedFirstFrameAttemptToken.parse('abcdefghijklmnopqrstu+'),
      throwsFormatException,
    );
    expect(
      () => RenderedFirstFrameAttemptToken.parse('abcdefghijklmnopqrstuv'),
      throwsFormatException,
    );
  });
}
