import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/data/smart_video_inventory.dart';

import '../support/fake_video_inventory.dart';
import '../support/scoped_video_media.dart';

void main() {
  test('prefetches future videos in order within the concurrency limit',
      () async {
    final store = FakeVideoCacheStore();
    final inventory = SmartVideoInventory(
      store: store,
      maxParallelDownloads: 2,
      maxPreparedVideos: 3,
    );
    final media = List.generate(
      4,
      (index) => scopedVideoMedia('https://media.test/$index.mp4'),
    );

    inventory.prepare(media);
    await Future<void>.delayed(Duration.zero);

    expect(store.downloads, media.take(2).map((item) => item.remoteUrl));
    expect(store.maximumActiveDownloads, 2);

    store.complete(media.first.remoteUrl!);
    await Future<void>.delayed(Duration.zero);

    expect(store.downloads, media.take(3).map((item) => item.remoteUrl));
    expect(store.downloads, isNot(contains(media.last.remoteUrl)));
  });
}
