import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/data/smart_video_inventory.dart';

import '../support/fake_video_inventory.dart';

void main() {
  test('rejects non-positive cache-download parallelism', () {
    expect(
      () => SmartVideoInventory(
        store: FakeVideoCacheStore(),
        maxParallelDownloads: 0,
        maxPreparedVideos: 1,
      ),
      throwsRangeError,
    );
  });
}
