import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

void main() {
  test('handle normalizes @ and accepts 1-30 ASCII word characters', () {
    final metadata = ProfileMetadata.parse(
      displayName: 'Nora Relay',
      handle: '  @NORA_42  ',
    );
    final longest = ProfileMetadata.parse(
      displayName: 'Nora Relay',
      handle: List<String>.filled(30, 'n').join(),
    );

    expect(metadata.handle.value, 'nora_42');
    expect(longest.handle.value, hasLength(30));
    expect(metadata.toSummary(ProfileId.parse('npub1nora')).handle, '@nora_42');
    final invalid = [
      '',
      '@',
      '@two words',
      'nora-relay',
      '@@nora',
      List<String>.filled(31, 'n').join(),
    ];
    for (final value in invalid) {
      expect(
        () => ProfileMetadata.parse(displayName: 'Nora Relay', handle: value),
        throwsFormatException,
        reason: '"$value" is not an allowed handle',
      );
    }
  });
}
