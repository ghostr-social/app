import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';
import '../../integration_test/support/progressive_device_resources.dart';

void main() {
  test('device resources preserve per-media range semantics', () async {
    final resources = await ProgressiveDeviceResources.start(
      rangeSemanticsById: const {
        'next': ProgressiveOriginRangeSemantics.ignored,
      },
    );
    final client = HttpClient();
    addTearDown(() async {
      client.close(force: true);
      await resources.close();
    });

    final request = await client.getUrl(resources.origin.urlFor('next'));
    request.headers.set(HttpHeaders.rangeHeader, 'bytes=1024-2047');
    final response = await request.close();
    final bytes = await response.fold<int>(0, (sum, part) {
      return sum + part.length;
    });

    expect(response.statusCode, HttpStatus.ok);
    expect(response.headers.value(HttpHeaders.contentRangeHeader), isNull);
    expect(bytes, resources.origin.objectLength);
  });
}
