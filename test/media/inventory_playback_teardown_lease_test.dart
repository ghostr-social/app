import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/inventory_video_playback_port.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/fake_video_inventory.dart';
import '../support/fake_video_player_platform.dart';
import '../support/blocking_video_controller_disposer.dart';

void main() {
  testWidgets('holds the cache lease until native player teardown completes',
      (tester) async {
    final platform = FakeVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final disposer = BlockingVideoControllerDisposer();
    final inventory = FakeVideoInventory();
    final playback = InventoryVideoPlaybackPort(
      delegate: VideoPlayerPlaybackPort(controllerDisposer: disposer.call),
      inventory: inventory,
    );
    final remote = VideoMediaSource.remote('https://media.test/video.mp4');

    await tester.pumpWidget(MaterialApp(
      home: playback.buildSurface(media: remote, isActive: true),
    ));
    inventory.complete(remote.debugLabel, VideoMediaSource.local('/cache/a'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
    expect(inventory.activeLeaseCount, 1);
    expect(platform.calls, containsAll(<String>['create', 'play']));

    await tester.pumpWidget(const SizedBox());
    expect(disposer.started.isCompleted, isTrue);
    expect(inventory.activeLeaseCount, 1);

    disposer.release.complete();
    for (var index = 0; index < 10; index += 1) {
      await tester.pump();
    }
    expect(inventory.activeLeaseCount, 0);
  });
}
