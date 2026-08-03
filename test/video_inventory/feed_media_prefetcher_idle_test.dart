import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/domain/feed_media_prefetcher.dart';

import '../support/fake_video_inventory.dart';
import '../support/sample_data.dart';

void main() {
  test('a feed with no neighbours prepares nothing', () {
    final inventory = FakeVideoInventory();
    final prefetcher = FeedMediaPrefetcher(inventory: inventory);

    prefetcher.focus(const [], 0);
    prefetcher.focus([samplePost()], 0);

    expect(inventory.prepared, isEmpty);
  });
}
