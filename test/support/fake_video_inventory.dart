import 'dart:async';

import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_lease.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_priority.dart';
import 'package:ghostr/features/video_inventory/domain/video_inventory_port.dart';

export 'fake_video_cache_store.dart';

class FakeVideoInventory implements VideoInventoryPort {
  final List<List<VideoMediaSource>> prepared = [];
  final List<VideoCachePriority> priorities = [];
  final Map<String, List<Completer<VideoCacheLease?>>> pending = {};
  int activeLeaseCount = 0;

  @override
  Future<VideoCacheLease?> acquire(
    VideoMediaSource media,
    VideoCachePriority priority,
  ) {
    priorities.add(priority);
    final waiter = Completer<VideoCacheLease?>();
    pending.putIfAbsent(media.debugLabel, () => []).add(waiter);
    return waiter.future;
  }

  @override
  void prepare(List<VideoMediaSource> media) => prepared.add(media);

  void complete(String label, VideoMediaSource media) {
    final waiters = pending[label]!;
    if (!media.isLocal) {
      for (final waiter in waiters) {
        waiter.complete();
      }
      return;
    }
    activeLeaseCount += 1;
    final lease = VideoCacheLease(media, () => activeLeaseCount -= 1);
    waiters.first.complete(lease);
    for (final waiter in waiters.skip(1)) {
      waiter.complete(lease.retain());
    }
  }

  void fail(String label) {
    for (final waiter in pending.remove(label)!) {
      waiter.completeError(StateError('cache unavailable'));
    }
  }
}
