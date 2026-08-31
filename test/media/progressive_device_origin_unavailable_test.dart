import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test(
    'unavailable origin records requests but serves no media body',
    () async {
      final origin = await ProgressiveDeviceOrigin.start(
        availability: ProgressiveOriginAvailability.unavailable,
      );
      addTearDown(origin.close);
      final client = HttpClient();
      addTearDown(client.close);

      final request = await client.getUrl(origin.urlFor('current'));
      final response = await request.close();
      final body = await response.fold<int>(0, (total, bytes) {
        return total + bytes.length;
      });

      expect(response.statusCode, HttpStatus.serviceUnavailable);
      expect(body, 0);
      expect(origin.bodyRequestedIds, ['current']);
      expect(origin.bytesServed('current'), 0);
    },
  );
}
