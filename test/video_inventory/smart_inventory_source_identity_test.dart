import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/data/smart_video_inventory.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_lease.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_priority.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_store.dart';

void main() {
  test('keeps shared-digest jobs separate when their source sets differ',
      () async {
    final store = _PendingCacheStore();
    final inventory = SmartVideoInventory(
      store: store,
      maxParallelDownloads: 2,
      maxPreparedVideos: 2,
    );
    final first = _media('https://one.test/video.mp4');
    final second = _media('https://two.test/video.mp4');

    final firstResult = inventory.acquire(first, VideoCachePriority.background);
    final secondResult =
        inventory.acquire(second, VideoCachePriority.background);
    await Future<void>.delayed(Duration.zero);

    expect(store.downloads, [first, second]);
    store.complete(0, '/cache/one.mp4');
    store.complete(1, '/cache/two.mp4');
    final firstLease = (await firstResult)!;
    final secondLease = (await secondResult)!;
    expect(firstLease.media.localPath, '/cache/one.mp4');
    expect(secondLease.media.localPath, '/cache/two.mp4');
    firstLease.release();
    secondLease.release();
  });
}

VideoMediaSource _media(String fallback) {
  return VideoMediaSource.withExpectedSha256(
    VideoMediaSource.remote(
      'https://media.test/video.mp4',
      fallbackUrls: [fallback],
    ),
    'e3b0c44298fc1c149afbf4c8996fb924'
    '27ae41e4649b934ca495991b7852b855',
  );
}

class _PendingCacheStore implements VideoCacheStore {
  final List<VideoMediaSource> downloads = [];
  final List<Completer<VideoCacheLease?>> _pending = [];

  @override
  Future<VideoCacheLease?> acquire(VideoMediaSource media) {
    downloads.add(media);
    final pending = Completer<VideoCacheLease?>();
    _pending.add(pending);
    return pending.future;
  }

  void complete(int index, String path) {
    final local = VideoMediaSource.local(path);
    _pending[index].complete(VideoCacheLease(local, () {}));
  }
}
