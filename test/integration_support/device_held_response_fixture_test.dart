import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/device_video_scenario.dart';
import '../../integration_test/support/device_video_server.dart';

void main() {
  test(
    'held-response fixture blocks a segment until explicit release',
    () async {
      final server = await DeviceVideoServer.start(
        DeviceVideoScenario.heldResponse,
      );
      addTearDown(server.close);
      final playlist = server.playbackUri('held-response');
      final segment = playlist.replace(
        path: playlist.path.replaceFirst('index.m3u8', 'index2.m4s'),
      );
      final client = HttpClient();
      addTearDown(() => client.close(force: true));

      final request = await client.getUrl(segment);
      final response = request.close();
      await _waitUntil(() => server.isResponseHeld);

      expect(server.heldResponses, 1);
      server.releaseHeldResponse();
      expect((await response).statusCode, HttpStatus.ok);
    },
  );
}

Future<void> _waitUntil(bool Function() condition) async {
  final watch = Stopwatch()..start();
  while (!condition() && watch.elapsed < const Duration(seconds: 2)) {
    await Future<void>.delayed(Duration.zero);
  }
  expect(condition(), isTrue);
}
