import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

void main() {
  test('normalizes non-empty profile identifiers and rejects blank input', () {
    expect(ProfileId.parse(' npub1creator ').value, 'npub1creator');
    expect(() => ProfileId.parse('  '), throwsFormatException);
  });
}
