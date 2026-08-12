import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';

void main() {
  test('display name trims and enforces its nonblank 50-character limit', () {
    final valid = ProfileMetadata.parse(
      displayName: '  Nora Relay  ',
      handle: 'nora',
    );
    final longest = ProfileMetadata.parse(
      displayName: List<String>.filled(50, 'n').join(),
      handle: 'nora',
    );

    expect(valid.displayName.value, 'Nora Relay');
    expect(longest.displayName.value, hasLength(50));
    expect(
      () => ProfileMetadata.parse(displayName: '   ', handle: 'nora'),
      throwsFormatException,
    );
    expect(
      () => ProfileMetadata.parse(
        displayName: List<String>.filled(51, 'n').join(),
        handle: 'nora',
      ),
      throwsFormatException,
    );
  });
}
