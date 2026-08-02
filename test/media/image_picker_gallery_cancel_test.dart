import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/image_picker_media_picker.dart';
import 'package:image_picker/image_picker.dart';

import '../support/fake_image_picker.dart';

void main() {
  test('returns no draft when gallery selection is cancelled', () async {
    final picker = FakeImagePicker();

    final media = await ImagePickerMediaPicker(picker).pickFromGallery();

    expect(picker.requestedSource, ImageSource.gallery);
    expect(media, isNull);
  });
}
