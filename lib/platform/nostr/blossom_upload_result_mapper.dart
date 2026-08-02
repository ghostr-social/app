import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_mime_type.dart';
import 'package:ghostr/features/publish/domain/uploaded_video_media.dart';
import 'package:ndk/ndk.dart';

class BlossomUploadResultMapper {
  const BlossomUploadResultMapper();

  static final _sha256Pattern = RegExp(r'^[0-9a-fA-F]{64}$');

  UploadedVideoMedia map(
    BlobUploadProgress progress, {
    required String fallbackMimeType,
  }) {
    final descriptors = _completedDescriptors(progress);
    if (!progress.isComplete || descriptors.isEmpty) {
      throw const AppFailure('No Blossom server accepted the video.');
    }
    final primary = descriptors.first;
    final mimeType = _mimeType(primary, fallbackMimeType);
    return UploadedVideoMedia(
      source: VideoMediaSource.remote(
        primary.url,
        fallbackUrls: descriptors.skip(1).map((item) => item.url).toList(),
      ),
      metadata: VideoUploadMetadata(
        sha256: primary.sha256,
        mimeType: mimeType.value,
        sizeBytes: primary.size!,
      ),
    );
  }

  List<BlobDescriptor> _completedDescriptors(BlobUploadProgress progress) {
    return progress.completedUploads
        .where((result) => result.success && result.descriptor != null)
        .map((result) => result.descriptor!)
        .where(_isValid)
        .toList();
  }

  VideoMimeType _mimeType(BlobDescriptor primary, String fallbackMimeType) {
    final mimeType = VideoMimeType.tryParse(primary.type) ??
        VideoMimeType.tryParse(fallbackMimeType);
    if (mimeType == null) {
      throw const AppFailure('The uploaded file is not a supported video.');
    }
    return mimeType;
  }

  bool _isValid(BlobDescriptor descriptor) {
    final uri = Uri.tryParse(descriptor.url);
    if (!_isHttpLocation(uri)) return false;
    if (!_sha256Pattern.hasMatch(descriptor.sha256)) return false;
    return (descriptor.size ?? 0) > 0;
  }

  bool _isHttpLocation(Uri? uri) {
    if (uri == null || uri.host.isEmpty) return false;
    return uri.scheme == 'https' || uri.scheme == 'http';
  }
}
