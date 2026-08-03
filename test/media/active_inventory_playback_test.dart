import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_priority.dart';
import 'package:ghostr/platform/media/inventory_video_playback_port.dart';

import '../support/fake_media_ports.dart';
import '../support/fake_video_inventory.dart';

void main() {
  testWidgets('prepares an active video before starting local playback',
      (tester) async {
    final inventory = FakeVideoInventory();
    final playback = InventoryVideoPlaybackPort(
      delegate: FakeVideoPlaybackPort(),
      inventory: inventory,
    );
    final remote = VideoMediaSource.remote('https://media.test/current.mp4');

    await tester.pumpWidget(MaterialApp(
      home: playback.buildSurface(media: remote, isActive: true),
    ));

    expect(find.bySemanticsLabel('Loading video'), findsOneWidget);
    expect(find.text(remote.debugLabel), findsNothing);
    expect(inventory.priorities, [VideoCachePriority.foreground]);

    inventory.complete(
      remote.debugLabel,
      VideoMediaSource.local('/cache/current.mp4'),
    );
    await tester.pump();

    expect(find.text('/cache/current.mp4'), findsOneWidget);
  });
}
