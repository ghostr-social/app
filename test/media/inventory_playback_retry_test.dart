import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/inventory_video_playback_port.dart';

import '../support/fake_media_ports.dart';
import '../support/fake_video_inventory.dart';

void main() {
  testWidgets('Retry recovers playback after cache preparation fails',
      (tester) async {
    final inventory = FakeVideoInventory();
    final playback = InventoryVideoPlaybackPort(
      delegate: FakeVideoPlaybackPort(),
      inventory: inventory,
    );
    final remote = VideoMediaSource.remote('https://media.test/retry.mp4');

    await tester.pumpWidget(MaterialApp(
      home: playback.buildSurface(media: remote, isActive: true),
    ));
    inventory.fail(remote.debugLabel);
    await tester.pump();

    await tester.tap(find.text('Retry'));
    await tester.pump();
    inventory.complete(
      remote.debugLabel,
      VideoMediaSource.local('/cache/retry.mp4'),
    );
    await tester.pump();

    expect(find.text('/cache/retry.mp4'), findsOneWidget);
    expect(find.text('Video unavailable'), findsNothing);
    expect(inventory.activeLeaseCount, 1);

    await tester.pumpWidget(const MaterialApp(home: SizedBox()));

    expect(inventory.activeLeaseCount, 0);
  });
}
