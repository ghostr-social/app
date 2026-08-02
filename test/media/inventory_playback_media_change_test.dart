import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/inventory_video_playback_port.dart';

import '../support/fake_media_ports.dart';
import '../support/fake_video_inventory.dart';

void main() {
  testWidgets('prepares replacement media when a feed card is reused',
      (tester) async {
    final inventory = FakeVideoInventory();
    final playback = InventoryVideoPlaybackPort(
      delegate: FakeVideoPlaybackPort(),
      inventory: inventory,
    );
    final first = VideoMediaSource.remote('https://media.test/first.mp4');
    final second = VideoMediaSource.remote('https://media.test/second.mp4');

    await tester.pumpWidget(MaterialApp(
      home: playback.buildSurface(media: first, isActive: false),
    ));
    await tester.pumpWidget(MaterialApp(
      home: playback.buildSurface(media: second, isActive: false),
    ));
    inventory.complete(
      second.debugLabel,
      VideoMediaSource.local('/cache/second.mp4'),
    );
    await tester.pump();
    await tester.pump();

    expect(find.text('/cache/second.mp4'), findsOneWidget);
    expect(inventory.pending.keys,
        containsAll([first.debugLabel, second.debugLabel]));
  });
}
