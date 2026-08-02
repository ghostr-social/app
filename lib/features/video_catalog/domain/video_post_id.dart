extension type const VideoPostId._(String value) implements String {
  factory VideoPostId.parse(String raw) {
    final value = raw.trim();
    if (value.isEmpty) {
      throw const FormatException('Video identifiers cannot be empty.');
    }
    return VideoPostId._(value);
  }
}
