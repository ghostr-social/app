import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/data/smart_video_inventory.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_priority.dart';

import '../support/fake_video_inventory.dart';
import '../support/scoped_video_media.dart';

void main() {
  test('shared playback requests retain one cache lease until both release',
      () async {
    final store = FakeVideoCacheStore();
    final inventory = SmartVideoInventory(
      store: store,
      maxParallelDownloads: 1,
      maxPreparedVideos: 1,
    );
    final media = scopedVideoMedia('https://media.test/video.mp4');

    final first = inventory.acquire(media, VideoCachePriority.foreground);
    final second = inventory.acquire(media, VideoCachePriority.foreground);
    await Future<void>.delayed(Duration.zero);
    store.complete(media.remoteUrl!);
    final firstLease = (await first)!;
    final secondLease = (await second)!;

    expect(store.activeLeaseCount, 1);
    firstLease.release();
    expect(store.activeLeaseCount, 1);
    secondLease.release();
    expect(store.activeLeaseCount, 0);
  });
}
