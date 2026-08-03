abstract interface class VideoFileDownloader {
  Future<void> download(
    Uri source,
    String destinationPath, {
    required int maxBytes,
    Duration? totalTimeout,
  });
}
