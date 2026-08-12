import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/image_picker_profile_image_picker.dart';

import '../support/fake_image_picker.dart';

void main() {
  test('canceling the photo library keeps the profile unchanged', () async {
    final selected = await ImagePickerProfileImagePicker(
      FakeImagePicker(),
    ).pickFromGallery();

    expect(selected, isNull);
  });
}
