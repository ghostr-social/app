import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/inventory_video_playback_port.dart';

import '../support/fake_media_ports.dart';
import '../support/fake_video_inventory.dart';

void main() {
  testWidgets('bypasses the inventory for an existing local video',
      (tester) async {
    final inventory = FakeVideoInventory();
    final playback = InventoryVideoPlaybackPort(
      delegate: FakeVideoPlaybackPort(),
      inventory: inventory,
    );
    final local = VideoMediaSource.local('/draft/video.mp4');

    await tester.pumpWidget(MaterialApp(
      home: playback.buildSurface(media: local, isActive: true),
    ));

    expect(find.text(local.debugLabel), findsOneWidget);
    expect(inventory.priorities, isEmpty);
  });
}
