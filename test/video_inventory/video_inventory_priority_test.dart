import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/data/smart_video_inventory.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_priority.dart';

import '../support/fake_video_inventory.dart';

void main() {
  test('moves the newly active video ahead of queued prefetch work', () async {
    final store = FakeVideoCacheStore();
    final inventory = SmartVideoInventory(
      store: store,
      maxParallelDownloads: 1,
      maxPreparedVideos: 3,
    );
    final first = VideoMediaSource.remote('https://media.test/first.mp4');
    final second = VideoMediaSource.remote('https://media.test/second.mp4');
    final active = VideoMediaSource.remote('https://media.test/active.mp4');

    inventory.prepare([first, second, active]);
    await Future<void>.delayed(Duration.zero);
    final activeResult = inventory.cache(active, VideoCachePriority.foreground);
    store.complete(first.remoteUrl!);
    await Future<void>.delayed(Duration.zero);

    expect(store.downloads, [first.remoteUrl, active.remoteUrl]);

    store.complete(active.remoteUrl!, path: '/cache/active.mp4');
    expect((await activeResult).localPath, '/cache/active.mp4');
  });
}
