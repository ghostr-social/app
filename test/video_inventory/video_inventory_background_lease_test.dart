import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/data/smart_video_inventory.dart';

import '../support/fake_video_inventory.dart';
import '../support/scoped_video_media.dart';

void main() {
  test('background preparation releases its cache handoff lease', () async {
    final store = FakeVideoCacheStore();
    final inventory = SmartVideoInventory(
      store: store,
      maxParallelDownloads: 1,
      maxPreparedVideos: 1,
    );
    final media = scopedVideoMedia('https://media.test/video.mp4');

    inventory.prepare([media]);
    await Future<void>.delayed(Duration.zero);
    store.complete(media.remoteUrl!);
    await Future<void>.delayed(Duration.zero);

    expect(store.activeLeaseCount, 0);
  });
}
