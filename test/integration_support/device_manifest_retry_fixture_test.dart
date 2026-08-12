import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/device_video_scenario.dart';
import '../../integration_test/support/device_video_server.dart';

void main() {
  test('manifest fixture fails once then serves the same URL', () async {
    final server = await DeviceVideoServer.start(
      DeviceVideoScenario.manifestRetry,
    );
    addTearDown(server.close);
    final uri = server.playbackUri('manifest-retry');

    expect(await _status(uri), HttpStatus.serviceUnavailable);
    expect(await _status(uri), HttpStatus.ok);

    expect(server.manifestFailures, 1);
    expect(server.successfulManifestResponses, 1);
    expect(server.requestsFor('index.m3u8'), 2);
  });
}

Future<int> _status(Uri uri) async {
  final client = HttpClient();
  try {
    final request = await client.getUrl(uri);
    final response = await request.close();
    await response.drain<void>();
    return response.statusCode;
  } finally {
    client.close(force: true);
  }
}
