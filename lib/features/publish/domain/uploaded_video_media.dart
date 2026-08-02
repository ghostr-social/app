import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_mime_type.dart';

class UploadedVideoMedia {
  factory UploadedVideoMedia({
    required VideoMediaSource source,
    required VideoUploadMetadata metadata,
  }) {
    final primaryUrl = source.remoteUrl;
    if (primaryUrl == null) {
      throw const FormatException('Uploaded video media must be remote.');
    }
    return UploadedVideoMedia._(
      primaryUrl,
      List.unmodifiable(source.fallbackUrls),
      metadata,
    );
  }

  const UploadedVideoMedia._(
    this.primaryUrl,
    this.fallbackUrls,
    this.metadata,
  );

  final String primaryUrl;
  final List<String> fallbackUrls;
  final VideoUploadMetadata metadata;

  String get sha256 => metadata.sha256;

  String get mimeType => metadata.mimeType;

  int get sizeBytes => metadata.sizeBytes;
}

class VideoUploadMetadata {
  factory VideoUploadMetadata({
    required String sha256,
    required String mimeType,
    required int sizeBytes,
  }) {
    final digest = sha256.trim().toLowerCase();
    final parsedMimeType = VideoMimeType.tryParse(mimeType);
    if (!RegExp(r'^[0-9a-f]{64}$').hasMatch(digest)) {
      throw const FormatException('A SHA-256 video digest is required.');
    }
    if (parsedMimeType == null || sizeBytes <= 0) {
      throw const FormatException('Uploaded video metadata is invalid.');
    }
    return VideoUploadMetadata._(
      digest,
      parsedMimeType.value,
      sizeBytes,
    );
  }

  const VideoUploadMetadata._(
    this.sha256,
    this.mimeType,
    this.sizeBytes,
  );

  final String sha256;
  final String mimeType;
  final int sizeBytes;
}
