import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test('activated staging times out when neither request arrives', () async {
    final origin = await ProgressiveDeviceOrigin.start();
    final observation = origin.stageFirstChunks({
      '/next.mp4',
      '/third.mp4',
    }, timeout: const Duration(milliseconds: 20));
    addTearDown(origin.close);

    observation.activate();
    await observation.settled.timeout(const Duration(seconds: 1));

    expect(observation.timedOut, isTrue);
    expect(observation.arrivedPaths, isEmpty);
  });
}
