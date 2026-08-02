import 'package:ghostr/core/media/video_mime_type.dart';

enum MediaPickSource { gallery, camera }

class SelectedMedia {
  factory SelectedMedia({
    required String path,
    required MediaPickSource source,
    required String label,
    required VideoMimeType mimeType,
  }) {
    return SelectedMedia._(
      path: _required(path, 'Video path'),
      source: source,
      label: _required(label, 'Video label'),
      mimeType: mimeType,
    );
  }

  const SelectedMedia._({
    required this.path,
    required this.source,
    required this.label,
    required this.mimeType,
  });

  final String path;
  final MediaPickSource source;
  final String label;
  final VideoMimeType mimeType;
}

String _required(String raw, String label) {
  final value = raw.trim();
  if (value.isEmpty) throw FormatException('$label is required.');
  return value;
}
