import 'package:ghostr/core/media/video_sha256.dart';

/// Blossom-style hosts address a file by its digest, so a URL whose last
/// path segment is a 64-hex name identifies the same file on any host.
VideoSha256? inferVideoSha256FromUrl(String url) {
  final segments = Uri.tryParse(url)?.pathSegments ?? const <String>[];
  if (segments.isEmpty) return null;
  final name = segments.last;
  final dot = name.indexOf('.');
  final stem = dot < 0 ? name : name.substring(0, dot);
  return VideoSha256.tryParse(stem);
}
