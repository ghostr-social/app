import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/media/image_picker_media_picker.dart';

import '../support/fake_image_picker.dart';

void main() {
  test('translates a denied picker request into an app-safe failure', () async {
    final picker = FakeImagePicker(
      error: PlatformException(code: 'camera_access_denied'),
    );

    final future = ImagePickerMediaPicker(picker).captureVideo();

    await expectLater(
      future,
      throwsA(isA<AppFailure>().having(
        (failure) => failure.message,
        'message',
        contains('Camera'),
      )),
    );
  });
}
