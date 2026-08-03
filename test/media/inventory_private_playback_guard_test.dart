import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/inventory_video_playback_port.dart';

import '../support/fake_media_ports.dart';
import '../support/fake_video_inventory.dart';

void main() {
  testWidgets('never delegates an uncached relay URL to playback',
      (tester) async {
    final inventory = FakeVideoInventory();
    final playback = InventoryVideoPlaybackPort(
      delegate: FakeVideoPlaybackPort(),
      inventory: inventory,
    );
    final media = VideoMediaSource.remote('http://127.0.0.1/admin.mp4');

    await tester.pumpWidget(MaterialApp(
      home: playback.buildSurface(media: media, isActive: true),
    ));

    expect(find.text(media.debugLabel), findsNothing);

    inventory.complete(media.debugLabel, media);
    await tester.pump();

    expect(find.text(media.debugLabel), findsNothing);
    expect(find.text('Video unavailable'), findsOneWidget);
  });
}
