import 'package:ghostr/core/media/video_media_source.dart';

abstract interface class VideoCacheStore {
  Future<VideoMediaSource?> find(VideoMediaSource media);

  Future<VideoMediaSource?> download(VideoMediaSource media);
}
