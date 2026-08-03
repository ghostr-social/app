import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/data/smart_video_inventory.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_priority.dart';

import '../support/fake_video_inventory.dart';

void main() {
  test('does not cache unpinned media without provenance', () async {
    final store = FakeVideoCacheStore();
    final inventory = SmartVideoInventory(
      store: store,
      maxParallelDownloads: 1,
      maxPreparedVideos: 1,
    );
    final media = VideoMediaSource.remote('https://media.test/mutable.mp4');

    final result = await inventory.acquire(
      media,
      VideoCachePriority.foreground,
    );

    expect(result, isNull);
    expect(store.downloads, isEmpty);
  });
}
