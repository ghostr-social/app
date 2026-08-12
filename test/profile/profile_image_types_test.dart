import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/domain/selected_profile_image.dart';

void main() {
  test('accepts supported local profile images and rejects other media', () {
    final image = SelectedProfileImage(
      path: ' /tmp/avatar.webp ',
      label: ' avatar.webp ',
      mimeType: ProfileImageMimeType.fromFileName('avatar.webp'),
    );

    expect(image.path, '/tmp/avatar.webp');
    expect(image.label, 'avatar.webp');
    expect(image.mimeType.value, 'image/webp');
    expect(
      () => ProfileImageMimeType.parse('video/mp4'),
      throwsFormatException,
    );
    expect(
      () => ProfileImageMimeType.fromFileName('avatar.txt'),
      throwsFormatException,
    );
  });
}
