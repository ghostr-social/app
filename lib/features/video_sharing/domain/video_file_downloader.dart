import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_sharing/domain/shareable_video_file.dart';

abstract interface class VideoFileDownloader {
  Future<ShareableVideoFile> download(VideoMediaSource media);
}
