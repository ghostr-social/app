import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/platform/media/image_picker_media_picker.dart';
import 'package:image_picker/image_picker.dart';

import '../support/fake_image_picker.dart';

void main() {
  test('maps a captured video into trusted selected media', () async {
    final picker = FakeImagePicker(
      result: XFile(
        '/tmp/clip.mov',
        name: 'clip.mov',
        mimeType: 'video/quicktime',
      ),
    );

    final media = await ImagePickerMediaPicker(picker).captureVideo();

    expect(picker.requestedSource, ImageSource.camera);
    expect(media?.source, MediaPickSource.camera);
    expect(media?.mimeType.value, 'video/quicktime');
  });
}
