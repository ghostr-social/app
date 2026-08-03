final class VideoMediaCacheScope {
  factory VideoMediaCacheScope.parse(String raw) {
    final value = raw.trim();
    if (value.isEmpty || value.length > 256) {
      throw const FormatException('A valid video cache scope is required.');
    }
    return VideoMediaCacheScope._(value);
  }

  const VideoMediaCacheScope._(this.value);

  final String value;
}
