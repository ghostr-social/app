import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/inventory_video_playback_port.dart';

import '../support/fake_media_ports.dart';
import '../support/fake_video_inventory.dart';

void main() {
  testWidgets('starts an already prepared local video when it becomes active',
      (tester) async {
    final inventory = FakeVideoInventory();
    final playback = InventoryVideoPlaybackPort(
      delegate: FakeVideoPlaybackPort(),
      inventory: inventory,
    );
    final remote = VideoMediaSource.remote('https://media.test/next.mp4');

    await tester.pumpWidget(MaterialApp(
      home: playback.buildSurface(media: remote, isActive: false),
    ));
    inventory.complete(
      remote.debugLabel,
      VideoMediaSource.local('/cache/next.mp4'),
    );
    await tester.pump();
    await tester.pumpWidget(MaterialApp(
      home: playback.buildSurface(media: remote, isActive: true),
    ));

    expect(find.text('/cache/next.mp4'), findsOneWidget);
  });
}
