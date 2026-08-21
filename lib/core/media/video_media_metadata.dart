/// Free metadata published alongside a remote video (NIP-92 `imeta` `size`
/// bytes and `duration` seconds) so delivery can plan before any probe.
class VideoMediaMetadata {
  const VideoMediaMetadata({this.sizeBytes, this.durationMs, this.blurhash});

  /// Lenient parse of raw imeta values: malformed entries become null.
  factory VideoMediaMetadata.fromImeta({
    String? size,
    String? duration,
    String? blurhash,
  }) {
    return VideoMediaMetadata(
      sizeBytes: _sizeBytes(size),
      durationMs: _durationMs(duration),
      blurhash: blurhash,
    );
  }

  static const VideoMediaMetadata none = VideoMediaMetadata();

  final int? sizeBytes;
  final int? durationMs;
  final String? blurhash;

  static int? _sizeBytes(String? raw) {
    final bytes = raw == null ? null : int.tryParse(raw.trim());
    return bytes == null || bytes <= 0 ? null : bytes;
  }

  static int? _durationMs(String? raw) {
    final seconds = raw == null ? null : num.tryParse(raw.trim());
    if (seconds == null || !seconds.isFinite || seconds <= 0) return null;
    return (seconds * 1000).round();
  }

  @override
  bool operator ==(Object other) {
    return other is VideoMediaMetadata &&
        other.sizeBytes == sizeBytes &&
        other.durationMs == durationMs &&
        other.blurhash == blurhash;
  }

  @override
  int get hashCode => Object.hash(sizeBytes, durationMs, blurhash);
}
