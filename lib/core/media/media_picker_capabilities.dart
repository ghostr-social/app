final class MediaPickerCapabilities {
  const MediaPickerCapabilities({
    required this.library,
    required this.camera,
  });

  const MediaPickerCapabilities.allSupported()
      : library = true,
        camera = true;

  const MediaPickerCapabilities.noneSupported()
      : library = false,
        camera = false;

  final bool library;
  final bool camera;

  @override
  bool operator ==(Object other) {
    return other is MediaPickerCapabilities &&
        other.library == library &&
        other.camera == camera;
  }

  @override
  int get hashCode => Object.hash(library, camera);
}
