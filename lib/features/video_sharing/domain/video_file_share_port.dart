import 'package:ghostr/features/video_sharing/domain/shareable_video_file.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_origin.dart';

abstract interface class VideoFileSharePort {
  Future<void> share(
    ShareableVideoFile file, {
    required VideoShareOrigin origin,
  });
}
