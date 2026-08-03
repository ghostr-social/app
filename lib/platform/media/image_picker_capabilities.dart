import 'package:flutter/foundation.dart';
import 'package:ghostr/core/media/media_picker_capabilities.dart';

MediaPickerCapabilities currentImagePickerCapabilities() {
  return imagePickerCapabilities(
    isWeb: kIsWeb,
    platform: defaultTargetPlatform,
  );
}

MediaPickerCapabilities imagePickerCapabilities({
  required bool isWeb,
  required TargetPlatform platform,
}) {
  if (isWeb || platform == TargetPlatform.fuchsia) {
    return const MediaPickerCapabilities.noneSupported();
  }
  final camera =
      platform == TargetPlatform.android || platform == TargetPlatform.iOS;
  return MediaPickerCapabilities(library: true, camera: camera);
}
