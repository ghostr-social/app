import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test('malformed range returns invalid 206 semantics', () async {
    final origin = await ProgressiveDeviceOrigin.start(
      rangeSemantics: ProgressiveOriginRangeSemantics.malformed,
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

    expect(response.statusCode, HttpStatus.partialContent);
    expect(response.headers.value(HttpHeaders.contentRangeHeader), 'invalid');
    expect(body, 1024);
    expect(origin.requests.single.range, (start: 1024, end: 2048));
  });
}
