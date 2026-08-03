import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/remote_video_delivery_source.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_playback_capabilities.dart';
import 'package:ghostr/features/video_catalog/data/playable_remote_video_source.dart';
import 'package:ghostr/features/video_inventory/data/inventory_remote_video_source.dart';

import '../support/fake_remote_video_source.dart';
import '../support/fake_video_inventory.dart';
import '../support/sample_data.dart';

void main() {
  test('unsupported primary media cannot hide a playable fallback', () async {
    final progressive = samplePost(id: 'progressive');
    final hls = samplePost(id: 'hls').withMedia(VideoMediaSource.remote(
      'https://media.example/live.m3u8',
      delivery: VideoMediaDelivery.hls,
    ));
    final combined = buildRemoteVideoDeliverySource(
      primary: PlayableRemoteVideoSource(
        source: FakeRemoteVideoSource([hls]),
        capabilities: VideoPlaybackCapabilities.progressiveOnly,
      ),
      nativeFallback: PlayableRemoteVideoSource(
        source: FakeRemoteVideoSource([progressive]),
        capabilities: VideoPlaybackCapabilities.progressiveOnly,
      ),
    );
    final inventory = FakeVideoInventory();
    final source = InventoryRemoteVideoSource(
      source: combined,
      inventory: inventory,
    );

    final posts = await source.loadRemoteFeed();

    expect(posts, [progressive]);
    expect(inventory.prepared, [
      [progressive.media],
    ]);
  });
}
