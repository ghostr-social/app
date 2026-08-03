import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/inventory_video_playback_port.dart';

import '../support/fake_media_ports.dart';
import '../support/fake_video_inventory.dart';

void main() {
  testWidgets('releases cached playback when its surface is disposed',
      (tester) async {
    final inventory = FakeVideoInventory();
    final playback = InventoryVideoPlaybackPort(
      delegate: FakeVideoPlaybackPort(),
      inventory: inventory,
    );
    final remote = VideoMediaSource.remote('https://media.test/video.mp4');

    await tester.pumpWidget(MaterialApp(
      home: playback.buildSurface(media: remote, isActive: true),
    ));
    inventory.complete(remote.debugLabel, VideoMediaSource.local('/cache/a'));
    await tester.pump();
    expect(inventory.activeLeaseCount, 1);

    await tester.pumpWidget(const MaterialApp(home: SizedBox()));

    expect(inventory.activeLeaseCount, 0);
  });
}
