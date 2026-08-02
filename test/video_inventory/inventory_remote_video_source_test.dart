import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/data/inventory_remote_video_source.dart';

import '../support/fake_remote_video_source.dart';
import '../support/fake_video_inventory.dart';
import '../support/sample_data.dart';

void main() {
  test('prepares relay videos in their feed order', () async {
    final posts = [samplePost(id: 'first'), samplePost(id: 'second')];
    final inventory = FakeVideoInventory();
    final source = InventoryRemoteVideoSource(
      source: FakeRemoteVideoSource(posts),
      inventory: inventory,
    );

    final loaded = await source.loadRemoteFeed();

    expect(loaded, same(posts));
    expect(inventory.prepared.single, posts.map((post) => post.media));
  });
}
