import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:video_player/video_player.dart';

import 'support/device_video_scenario.dart';
import 'support/device_video_server.dart';
import 'support/video_player_contract_wait.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('locked plugin exposes separate stall start and end values', (
    tester,
  ) async {
    final server = await DeviceVideoServer.start(
      DeviceVideoScenario.heldResponse,
    );
    final controller = VideoPlayerController.networkUrl(
      server.playbackUri('stall'),
      formatHint: VideoFormat.hls,
    );
    final transitions = <bool>[];
    bool? previous;
    controller.addListener(() {
      final buffering = controller.value.isBuffering;
      if (controller.value.isInitialized && buffering != previous) {
        transitions.add(buffering);
        previous = buffering;
      }
    });
    addTearDown(server.close);

    await controller.initialize();
    await mountContractPlayer(tester, controller);
    await controller.play();
    await waitForController(
      tester,
      controller,
      (value) => value.position >= const Duration(milliseconds: 300),
    );
    await waitForController(tester, controller, (value) => value.isBuffering);
    final stallStart = transitions.lastIndexOf(true);

    server.releaseHeldResponse();
    await waitForController(tester, controller, (value) => !value.isBuffering);

    expect(stallStart, greaterThanOrEqualTo(0));
    expect(transitions.indexOf(false, stallStart + 1), greaterThan(stallStart));
    await controller.dispose();
  });
}
