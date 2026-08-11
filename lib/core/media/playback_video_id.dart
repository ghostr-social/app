final class PlaybackVideoId {
  factory PlaybackVideoId.parse(String raw) {
    final value = raw.trim();
    if (value.isEmpty) {
      throw const FormatException('A playback video id is required.');
    }
    return PlaybackVideoId._(value);
  }

  const PlaybackVideoId._(this.value);

  final String value;

  @override
  bool operator ==(Object other) {
    return other is PlaybackVideoId && other.value == value;
  }

  @override
  int get hashCode => value.hashCode;
}
