import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/media/media_picker_capabilities.dart';
import 'package:ghostr/platform/media/image_picker_media_picker.dart';

import '../support/fake_image_picker.dart';

void main() {
  test('rejects camera capture before calling the desktop picker', () async {
    final picker = FakeImagePicker();
    final adapter = ImagePickerMediaPicker(
      picker,
      capabilities: const MediaPickerCapabilities(library: true, camera: false),
    );

    await expectLater(
      adapter.captureVideo(),
      throwsA(isA<AppFailure>().having(
        (failure) => failure.message,
        'message',
        contains('unavailable'),
      )),
    );
    expect(picker.requestedSource, isNull);
  });
}
