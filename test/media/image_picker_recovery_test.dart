import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/platform/media/image_picker_media_picker.dart';
import 'package:image_picker/image_picker.dart';

import '../support/fake_image_picker.dart';

void main() {
  test('maps a video recovered after Android activity destruction', () async {
    final picker = FakeImagePicker(
      lostData: LostDataResponse(
        file: XFile('/tmp/recovered.mp4', name: 'recovered.mp4'),
        type: RetrieveType.video,
      ),
    );

    final media = await ImagePickerMediaPicker(picker).recoverLostVideo();

    expect(media?.source, MediaPickSource.gallery);
    expect(media?.path, '/tmp/recovered.mp4');
  });
}
