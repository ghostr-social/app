import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_event_mapper.dart';
import 'package:ghostr/features/video_inventory/data/smart_video_inventory.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_priority.dart';
import 'package:ndk/ndk.dart';

import '../support/fake_video_inventory.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('plays HLS remotely without caching its playlist as a video file',
      () async {
    final media = const NostrVideoEventMapper().map(_hlsEvent(), null).media;
    final store = FakeVideoCacheStore();
    final inventory = SmartVideoInventory(
      store: store,
      maxParallelDownloads: 1,
      maxPreparedVideos: 1,
    );

    inventory.prepare([media]);
    await Future<void>.delayed(Duration.zero);

    expect(store.downloads, isEmpty);
    expect(
      await inventory.cache(media, VideoCachePriority.foreground),
      same(media),
    );
  });
}

Nip01Event _hlsEvent() {
  return Nip01Event(
    id: testEventId,
    pubKey: testCreatorPublicKey,
    kind: 34236,
    createdAt: 20,
    content: 'Adaptive stream',
    tags: const [
      ['d', 'hls-video'],
      [
        'imeta',
        'url https://media.example/stream',
        'm application/x-mpegURL',
      ],
    ],
  );
}
