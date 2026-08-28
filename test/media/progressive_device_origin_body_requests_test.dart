import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test(
    'body-requested ids exclude metadata probes and preserve order',
    () async {
      final origin = await ProgressiveDeviceOrigin.start();
      addTearDown(origin.close);
      origin.requests.addAll([
        _request('HEAD', '/current.mp4'),
        _request('GET', '/next.mp4'),
        _request('GET', '/third.mp4'),
        _request('GET', '/next.mp4'),
      ]);

      expect(origin.bodyRequestedIds, ['next', 'third']);
    },
  );
}

ProgressiveOriginRequest _request(String method, String path) {
  return ProgressiveOriginRequest(method, path, null, startedAt: Duration.zero);
}
