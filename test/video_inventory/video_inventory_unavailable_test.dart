import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/data/smart_video_inventory.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_priority.dart';

import '../support/fake_video_inventory.dart';

void main() {
  test('keeps the remote source when the store cannot retain a download',
      () async {
    final store = FakeVideoCacheStore();
    final inventory = SmartVideoInventory(
      store: store,
      maxParallelDownloads: 1,
      maxPreparedVideos: 1,
    );
    final remote = VideoMediaSource.remote('https://media.test/video.mp4');

    final result = inventory.cache(remote, VideoCachePriority.foreground);
    await Future<void>.delayed(Duration.zero);
    store.completeUnavailable(remote.remoteUrl!);

    expect(await result, same(remote));
  });
}
