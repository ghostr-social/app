part of 'video_media_source.dart';

String _localPath(String raw) {
  final path = raw.trim();
  if (path.isEmpty) throw const FormatException('Video path is required.');
  return path;
}

String _httpUrl(String raw) {
  final value = raw.trim();
  final uri = Uri.tryParse(value);
  final isHttp = uri?.scheme == 'https' || uri?.scheme == 'http';
  if (uri == null || !isHttp || uri.host.isEmpty) {
    throw FormatException('Invalid video URL: $raw');
  }
  return value;
}
