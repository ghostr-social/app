import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/inventory_video_playback_port.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/blocking_video_controller_disposer.dart';
import '../support/fake_video_inventory.dart';
import '../support/fake_video_player_platform.dart';

void main() {
  testWidgets('keeps the replaced lease until its player closes',
      (tester) async {
    final platform = FakeVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final disposer = BlockingVideoControllerDisposer();
    final inventory = FakeVideoInventory();
    final playback = InventoryVideoPlaybackPort(
      delegate: VideoPlayerPlaybackPort(controllerDisposer: disposer.call),
      inventory: inventory,
    );
    final first = VideoMediaSource.remote('https://media.test/first.mp4');
    final second = VideoMediaSource.remote('https://media.test/second.mp4');

    await tester.pumpWidget(MaterialApp(
      home: playback.buildSurface(media: first, isActive: true),
    ));
    inventory.complete(first.debugLabel, VideoMediaSource.local('/cache/a'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));

    await tester.pumpWidget(MaterialApp(
      home: playback.buildSurface(media: second, isActive: true),
    ));
    inventory.complete(second.debugLabel, VideoMediaSource.local('/cache/b'));
    await tester.pump();
    expect(disposer.started.isCompleted, isTrue);
    expect(inventory.activeLeaseCount, 2);

    disposer.release.complete();
    await tester.pump();
    expect(inventory.activeLeaseCount, 1);
    await tester.pump(const Duration(milliseconds: 100));

    await tester.pumpWidget(const MaterialApp(home: SizedBox()));
    for (var index = 0; index < 5; index += 1) {
      await tester.pump();
    }
    expect(inventory.activeLeaseCount, 0);
  });
}
