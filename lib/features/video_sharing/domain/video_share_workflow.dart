import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_origin.dart';

abstract interface class VideoShareWorkflow {
  bool supports(VideoMediaSource media);

  Future<void> share(
    VideoMediaSource media, {
    required VideoShareOrigin origin,
  });
}
