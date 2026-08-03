class VideoDownloadTimeouts {
  const VideoDownloadTimeouts({
    required this.headers,
    required this.idle,
    required this.total,
  });

  static const defaults = VideoDownloadTimeouts(
    headers: Duration(seconds: 15),
    idle: Duration(seconds: 15),
    total: Duration(minutes: 5),
  );

  final Duration headers;
  final Duration idle;
  final Duration total;
}
