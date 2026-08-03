import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/domain/feed_media_prefetcher.dart';

import '../support/fake_video_inventory.dart';
import '../support/sample_data.dart';

void main() {
  test('prefetches the window around the active video, ahead first', () {
    final inventory = FakeVideoInventory();
    final prefetcher = FeedMediaPrefetcher(
      inventory: inventory,
      ahead: 3,
      behind: 1,
    );
    final posts = [
      for (var index = 0; index < 6; index += 1) samplePost(id: 'post-$index'),
    ];

    prefetcher.focus(posts, 4);

    expect(
      inventory.prepared.single.map((media) => media.remoteUrl),
      [
        'https://example.com/video/post-5.mp4',
        'https://example.com/video/post-3.mp4',
      ],
    );
  });
}
