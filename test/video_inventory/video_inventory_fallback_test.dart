import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/data/smart_video_inventory.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_priority.dart';

import '../support/fake_video_inventory.dart';
import '../support/scoped_video_media.dart';

void main() {
  test('returns no playback lease when a cache download fails', () async {
    final store = FakeVideoCacheStore();
    final inventory = SmartVideoInventory(
      store: store,
      maxParallelDownloads: 1,
      maxPreparedVideos: 1,
    );
    final remote = scopedVideoMedia('https://media.test/video.mp4');

    final result = inventory.acquire(remote, VideoCachePriority.foreground);
    await Future<void>.delayed(Duration.zero);
    store.fail(remote.remoteUrl!);

    expect(await result, isNull);
  });
}
