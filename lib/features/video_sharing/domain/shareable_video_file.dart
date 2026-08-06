final class ShareableVideoFile {
  const ShareableVideoFile._(this.path);

  factory ShareableVideoFile.parse(String rawPath) {
    final path = rawPath.trim();
    final scheme = Uri.tryParse(path)?.scheme.toLowerCase();
    if (path.isEmpty || scheme == 'http' || scheme == 'https') {
      throw FormatException('A local video file path is required.');
    }
    return ShareableVideoFile._(path);
  }

  final String path;

  @override
  bool operator ==(Object other) {
    return other is ShareableVideoFile && other.path == path;
  }

  @override
  int get hashCode => path.hashCode;
}
