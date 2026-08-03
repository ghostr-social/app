import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/media_picker_capabilities.dart';
import 'package:ghostr/platform/media/image_picker_capabilities.dart';

void main() {
  test('models end-to-end picker support for every platform', () {
    for (final platform in [TargetPlatform.android, TargetPlatform.iOS]) {
      expect(
        imagePickerCapabilities(isWeb: false, platform: platform),
        const MediaPickerCapabilities.allSupported(),
      );
    }
    for (final platform in [
      TargetPlatform.linux,
      TargetPlatform.macOS,
      TargetPlatform.windows,
    ]) {
      expect(
        imagePickerCapabilities(isWeb: false, platform: platform),
        const MediaPickerCapabilities(library: true, camera: false),
      );
    }
    expect(
      imagePickerCapabilities(
        isWeb: true,
        platform: TargetPlatform.windows,
      ),
      const MediaPickerCapabilities.noneSupported(),
    );
    expect(
      currentImagePickerCapabilities(),
      imagePickerCapabilities(
        isWeb: kIsWeb,
        platform: defaultTargetPlatform,
      ),
    );
  });

  test('capability values compare and hash by supported source', () {
    final unsupported = MediaPickerCapabilities.noneSupported();
    final same = MediaPickerCapabilities(library: false, camera: false);

    expect(unsupported, same);
    expect(unsupported.hashCode, same.hashCode);
    expect(
      unsupported,
      isNot(const MediaPickerCapabilities.allSupported()),
    );
  });
}
