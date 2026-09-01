import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test('ignored range returns one complete 200 response', () async {
    final origin = await ProgressiveDeviceOrigin.start(
      rangeSemantics: ProgressiveOriginRangeSemantics.ignored,
    );
    final client = HttpClient();
    addTearDown(() async {
      client.close(force: true);
      await origin.close();
    });

    final request = await client.getUrl(origin.urlFor('current'));
    request.headers.set(HttpHeaders.rangeHeader, 'bytes=1024-2047');
    final response = await request.close();
    final body = await response.fold<int>(0, (total, bytes) {
      return total + bytes.length;
    });

    expect(response.statusCode, HttpStatus.ok);
    expect(response.headers.value(HttpHeaders.acceptRangesHeader), 'bytes');
    expect(response.headers.value(HttpHeaders.contentRangeHeader), isNull);
    expect(body, origin.objectLength);
    expect(origin.requests.single.range, (start: 1024, end: 2048));
    expect(origin.bytesServed('current'), origin.objectLength);
  });
}
