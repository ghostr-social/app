import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/domain/selected_profile_image.dart';

void main() {
  test('profile image selection requires a path and display label', () {
    final mimeType = ProfileImageMimeType.parse('image/png');

    expect(
      () => SelectedProfileImage(
        path: ' ',
        label: 'avatar.png',
        mimeType: mimeType,
      ),
      throwsFormatException,
    );
    expect(
      () => SelectedProfileImage(
        path: '/tmp/avatar.png',
        label: ' ',
        mimeType: mimeType,
      ),
      throwsFormatException,
    );
  });
}
