import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';

void main() {
  test('rejects credentials embedded in a public profile picture URL', () {
    expect(
      () => ProfileMetadata.parse(
        displayName: 'Nora',
        handle: 'nora',
        pictureUrl: 'https://nora:secret@example.com/avatar.jpg',
      ),
      throwsFormatException,
    );
  });
}
