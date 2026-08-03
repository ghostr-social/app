import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/inventory_video_playback_port.dart';

import '../support/fake_media_ports.dart';
import '../support/fake_video_inventory.dart';

void main() {
  testWidgets('labels HLS unsupported without offering a futile retry',
      (tester) async {
    final inventory = FakeVideoInventory();
    final playback = InventoryVideoPlaybackPort(
      delegate: FakeVideoPlaybackPort(),
      inventory: inventory,
    );
    final media = VideoMediaSource.remote(
      'https://media.test/live.m3u8',
      delivery: VideoMediaDelivery.hls,
    );

    await tester.pumpWidget(MaterialApp(
      home: playback.buildSurface(media: media, isActive: true),
    ));

    expect(find.text('Streaming video unsupported'), findsOneWidget);
    expect(find.text('Retry'), findsNothing);
    expect(inventory.priorities, isEmpty);
  });
}
