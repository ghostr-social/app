import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:video_player/video_player.dart';

import 'support/device_video_scenario.dart';
import 'support/device_video_server.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('locked plugin exposes initialization errors', (tester) async {
    final server = await DeviceVideoServer.start(DeviceVideoScenario.contract);
    final controller = VideoPlayerController.networkUrl(server.missingUri);
    addTearDown(server.close);

    await expectLater(controller.initialize(), throwsA(anything));

    expect(controller.value.hasError, isTrue);
    expect(controller.value.errorDescription, isNotEmpty);
    await controller.dispose();
  });
}
