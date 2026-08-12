extension type const ProfileImageMimeType._(String value) implements String {
  factory ProfileImageMimeType.parse(String raw) {
    final value = raw.trim().toLowerCase();
    if (!_supported.contains(value)) {
      throw const FormatException('Select a JPEG, PNG, GIF, or WebP image.');
    }
    return ProfileImageMimeType._(value);
  }

  static ProfileImageMimeType? tryParse(String? raw) {
    if (raw == null) return null;
    try {
      return ProfileImageMimeType.parse(raw);
    } on FormatException {
      return null;
    }
  }

  factory ProfileImageMimeType.fromFileName(String fileName) {
    final extension = fileName.trim().toLowerCase().split('.').lastOrNull;
    final mimeType = switch (extension) {
      'jpg' || 'jpeg' => 'image/jpeg',
      'png' => 'image/png',
      'gif' => 'image/gif',
      'webp' => 'image/webp',
      _ => '',
    };
    return ProfileImageMimeType.parse(mimeType);
  }
}

const _supported = {'image/jpeg', 'image/png', 'image/gif', 'image/webp'};

final class SelectedProfileImage {
  factory SelectedProfileImage({
    required String path,
    required String label,
    required ProfileImageMimeType mimeType,
  }) {
    return SelectedProfileImage._(
      _required(path, 'Image path'),
      _required(label, 'Image label'),
      mimeType,
    );
  }

  const SelectedProfileImage._(this.path, this.label, this.mimeType);

  final String path;
  final String label;
  final ProfileImageMimeType mimeType;
}

String _required(String raw, String label) {
  final value = raw.trim();
  if (value.isEmpty) throw FormatException('$label is required.');
  return value;
}
