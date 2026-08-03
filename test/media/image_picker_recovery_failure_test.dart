import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/media/media_picker_capabilities.dart';
import 'package:ghostr/platform/media/image_picker_media_picker.dart';
import 'package:image_picker/image_picker.dart';

import '../support/fake_image_picker.dart';

void main() {
  test('translates an interrupted picker recovery failure', () async {
    final picker = FakeImagePicker(
      lostData: LostDataResponse(
        exception: PlatformException(code: 'lost_video_unavailable'),
        type: RetrieveType.video,
      ),
    );

    await expectLater(
      ImagePickerMediaPicker(
        picker,
        capabilities:
            const MediaPickerCapabilities(library: true, camera: false),
      ).recoverLostVideo(),
      throwsA(isA<AppFailure>().having(
        (failure) => failure.message,
        'message',
        contains('interrupted'),
      )),
    );
  });
}
