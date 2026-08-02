class VideoMimeType {
  const VideoMimeType._(this.value);

  static const _byExtension = <String, String>{
    'mp4': 'video/mp4',
    'm4v': 'video/x-m4v',
    'mov': 'video/quicktime',
    'webm': 'video/webm',
    'mkv': 'video/x-matroska',
    '3gp': 'video/3gpp',
  };

  final String value;

  factory VideoMimeType.fromFileName(String fileName) {
    final extension = fileName.split('.').last.toLowerCase();
    final value = _byExtension[extension];
    if (value == null) throw FormatException('Unsupported video: $fileName');
    return VideoMimeType._(value);
  }

  static VideoMimeType? tryParse(String? raw) {
    final value = raw?.trim().toLowerCase();
    if (!_byExtension.containsValue(value)) return null;
    return VideoMimeType._(value!);
  }
}
