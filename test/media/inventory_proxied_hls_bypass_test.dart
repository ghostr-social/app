import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/inventory_video_playback_port.dart';

import '../support/fake_media_ports.dart';
import '../support/fake_video_inventory.dart';

void main() {
  testWidgets('bypasses file inventory only for a trusted HLS proxy',
      (tester) async {
    final inventory = FakeVideoInventory();
    final playback = InventoryVideoPlaybackPort(
      delegate: FakeVideoPlaybackPort(),
      inventory: inventory,
    );
    final media = VideoMediaSource.proxiedHls(
      'http://127.0.0.1:3210/hls/'
      '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef/'
      'index.m3u8',
    );

    await tester.pumpWidget(MaterialApp(
      home: playback.buildSurface(media: media, isActive: true),
    ));

    expect(find.text('Secure HLS stream'), findsOneWidget);
    expect(inventory.priorities, isEmpty);
  });
}
