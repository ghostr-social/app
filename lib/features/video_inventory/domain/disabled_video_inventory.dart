import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_lease.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_priority.dart';
import 'package:ghostr/features/video_inventory/domain/video_inventory_port.dart';

final class DisabledVideoInventory implements VideoInventoryPort {
  const DisabledVideoInventory();

  @override
  Future<VideoCacheLease?> acquire(
    VideoMediaSource media,
    VideoCachePriority priority,
  ) {
    return Future.value();
  }

  @override
  void prepare(List<VideoMediaSource> media) {}
}
