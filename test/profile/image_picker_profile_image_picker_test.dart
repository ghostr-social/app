import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/media/image_picker_profile_image_picker.dart';
import 'package:image_picker/image_picker.dart';

import '../support/fake_image_picker.dart';

void main() {
  test('selects an image and translates picker failures safely', () async {
    final picker = FakeImagePicker(
      result: XFile('/tmp/avatar.jpg', mimeType: 'image/jpeg'),
    );
    final selected = await ImagePickerProfileImagePicker(
      picker,
    ).pickFromGallery();

    expect(picker.requestedImageSource, ImageSource.gallery);
    expect(picker.requestedImageWidth, 1024);
    expect(picker.requestedImageHeight, 1024);
    expect(picker.requestedImageQuality, 85);
    expect(selected?.label, 'avatar.jpg');
    expect(selected?.mimeType.value, 'image/jpeg');

    final fallbackPicker = FakeImagePicker(
      result: XFile('/tmp/avatar.webp', mimeType: 'application/octet-stream'),
    );
    final fallback = await ImagePickerProfileImagePicker(
      fallbackPicker,
    ).pickFromGallery();
    expect(fallback?.mimeType.value, 'image/webp');

    final denied = FakeImagePicker(
      error: PlatformException(code: 'photo_access_denied'),
    );
    await expectLater(
      ImagePickerProfileImagePicker(denied).pickFromGallery(),
      throwsA(
        isA<AppFailure>().having(
          (failure) => failure.message,
          'message',
          contains('Photo library'),
        ),
      ),
    );
  });
}
