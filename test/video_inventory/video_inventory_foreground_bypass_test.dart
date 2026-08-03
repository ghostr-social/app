import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/data/smart_video_inventory.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_priority.dart';

import '../support/fake_video_cache_store.dart';

void main() {
  test('a foreground request starts even when downloads occupy every slot',
      () async {
    final store = FakeVideoCacheStore();
    final inventory = SmartVideoInventory(
      store: store,
      maxParallelDownloads: 1,
      maxPreparedVideos: 8,
    );
    inventory.prepare([_media('https://media.test/queued.mp4', '1')]);

    final foreground = inventory.acquire(
      _media('https://media.test/current.mp4', '2'),
      VideoCachePriority.foreground,
    );
    await pumpEventQueue();

    expect(
      store.downloads,
      ['https://media.test/queued.mp4', 'https://media.test/current.mp4'],
    );
    store.complete('https://media.test/current.mp4');
    store.complete('https://media.test/queued.mp4');
    final lease = await foreground;
    expect(lease, isNotNull);
    lease!.release();
  });
}

VideoMediaSource _media(String url, String seed) {
  return VideoMediaSource.withExpectedSha256(
    VideoMediaSource.remote(url),
    seed * 64,
  );
}
