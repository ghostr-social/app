import 'dart:math';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:video_player/video_player.dart';

import 'support/device_video_scenario.dart';
import 'support/device_video_server.dart';
import 'support/video_player_contract_wait.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('locked plugin prepares next with one audible player', (
    tester,
  ) async {
    final server = await DeviceVideoServer.start(DeviceVideoScenario.contract);
    final current = _controller(server, 'current');
    final next = _controller(server, 'next');
    addTearDown(server.close);
    await Future.wait([current.initialize(), next.initialize()]);
    await tester.pumpWidget(
      MaterialApp(
        home: Stack(children: [VideoPlayer(current), VideoPlayer(next)]),
      ),
    );
    var maxAudiblePlaying = 0;
    void audit() {
      final audible = [
        current,
        next,
      ].where((item) => item.value.isPlaying && item.value.volume > 0);
      maxAudiblePlaying = max(maxAudiblePlaying, audible.length);
    }

    current.addListener(audit);
    next.addListener(audit);
    await next.setVolume(0);
    await current.play();
    await waitForController(
      tester,
      current,
      (value) => value.position > Duration.zero,
    );
    await current.setVolume(0);
    await current.pause();
    await next.play();
    await next.setVolume(1);
    await waitForController(
      tester,
      next,
      (value) => value.position > Duration.zero,
    );

    expect(current.value.isInitialized, isTrue);
    expect(next.value.isInitialized, isTrue);
    expect(maxAudiblePlaying, lessThanOrEqualTo(1));
    await Future.wait([current.dispose(), next.dispose()]);
  });
}

VideoPlayerController _controller(DeviceVideoServer server, String id) {
  return VideoPlayerController.networkUrl(
    server.playbackUri(id),
    formatHint: VideoFormat.hls,
  );
}
