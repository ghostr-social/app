import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/domain/player_preparation_feedback_port.dart';

void main() {
  test('native attempt token is exactly 128-bit base64url', () {
    const valid = 'abcdefghijklmnopqrstuA';

    expect(PlayerPreparationAttemptToken.parse(valid).value, valid);
    expect(
      () => PlayerPreparationAttemptToken.parse('too-short'),
      throwsFormatException,
    );
    expect(
      () => PlayerPreparationAttemptToken.parse('abcdefghijklmnopqrstu+'),
      throwsFormatException,
    );
    expect(
      () => PlayerPreparationAttemptToken.parse('abcdefghijklmnopqrstuv'),
      throwsFormatException,
    );
  });
}
