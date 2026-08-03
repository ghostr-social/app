import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/inventory_video_playback_port.dart';

import '../support/fake_media_ports.dart';
import '../support/fake_video_inventory.dart';

void main() {
  testWidgets('releases cached playback when the surface changes media',
      (tester) async {
    final inventory = FakeVideoInventory();
    final playback = InventoryVideoPlaybackPort(
      delegate: FakeVideoPlaybackPort(),
      inventory: inventory,
    );
    final first = VideoMediaSource.remote('https://media.test/first.mp4');
    final second = VideoMediaSource.remote('https://media.test/second.mp4');

    await tester.pumpWidget(MaterialApp(
      home: playback.buildSurface(media: first, isActive: true),
    ));
    inventory.complete(first.debugLabel, VideoMediaSource.local('/cache/a'));
    await tester.pump();
    expect(inventory.activeLeaseCount, 1);

    await tester.pumpWidget(MaterialApp(
      home: playback.buildSurface(media: second, isActive: true),
    ));

    expect(inventory.activeLeaseCount, 0);
  });
}
