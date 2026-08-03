import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_lease.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_priority.dart';

abstract interface class VideoInventoryPort {
  void prepare(List<VideoMediaSource> media);

  Future<VideoCacheLease?> acquire(
    VideoMediaSource media,
    VideoCachePriority priority,
  );
}
