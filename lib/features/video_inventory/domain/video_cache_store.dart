import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_lease.dart';

abstract interface class VideoCacheStore {
  Future<VideoCacheLease?> acquire(VideoMediaSource media);
}
