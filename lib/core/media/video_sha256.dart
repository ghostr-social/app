final class VideoSha256 {
  factory VideoSha256.parse(String raw) {
    final digest = tryParse(raw);
    if (digest == null) {
      throw const FormatException('A SHA-256 video digest is required.');
    }
    return digest;
  }

  const VideoSha256._(this.value);

  static final _pattern = RegExp(r'^[0-9a-f]{64}$');

  final String value;

  static VideoSha256? tryParse(String raw) {
    final value = raw.trim().toLowerCase();
    return _pattern.hasMatch(value) ? VideoSha256._(value) : null;
  }
}
