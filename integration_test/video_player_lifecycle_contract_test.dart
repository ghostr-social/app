import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:video_player/video_player.dart';

import 'support/device_video_scenario.dart';
import 'support/device_video_server.dart';
import 'support/video_player_contract_wait.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('locked plugin exposes lifecycle and playback metrics', (
    tester,
  ) async {
    final server = await DeviceVideoServer.start(DeviceVideoScenario.contract);
    final controller = VideoPlayerController.networkUrl(
      server.playbackUri('lifecycle'),
      formatHint: VideoFormat.hls,
    );
    final values = <VideoPlayerValue>[];
    controller.addListener(() => values.add(controller.value));
    addTearDown(server.close);

    await controller.initialize();
    await mountContractPlayer(tester, controller);
    expect(controller.value.isInitialized, isTrue);

    await controller.setPlaybackSpeed(1.25);
    await controller.play();
    await waitForController(
      tester,
      controller,
      (value) =>
          value.position >= const Duration(milliseconds: 300) &&
          value.buffered.isNotEmpty,
    );
    expect(controller.value.playbackSpeed, 1.25);
    expect(controller.value.buffered, isNotEmpty);

    await controller.pause();
    expect(controller.value.isPlaying, isFalse);
    await controller.seekTo(const Duration(seconds: 2));
    await waitForController(
      tester,
      controller,
      (value) => value.position >= const Duration(seconds: 2),
    );

    await controller.setPlaybackSpeed(1);
    await controller.seekTo(
      controller.value.duration - const Duration(seconds: 1),
    );
    await controller.play();
    await waitForController(tester, controller, (value) => value.isCompleted);
    expect(values.any((value) => value.isCompleted), isTrue);

    await controller.dispose();
    final disposedCount = values.length;
    await pumpContractFrame(tester);
    expect(values, hasLength(disposedCount));
  });
}
