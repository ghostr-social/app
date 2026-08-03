import 'package:ghostr/core/media/video_media_source.dart';

VideoMediaSource scopedVideoMedia(String url) {
  return VideoMediaSource.withCacheScope(
    VideoMediaSource.remote(url),
    'test:$url',
  );
}
