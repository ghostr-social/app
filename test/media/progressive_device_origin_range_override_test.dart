import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test('range semantics can target one media id', () async {
    final origin = await ProgressiveDeviceOrigin.start(
      rangeSemanticsById: const {
        'next': ProgressiveOriginRangeSemantics.malformed,
      },
    );
    final client = HttpClient();
    addTearDown(() async {
      client.close(force: true);
      await origin.close();
    });

    final current = await _range(client, origin.urlFor('current'));
    final next = await _range(client, origin.urlFor('next'));

    expect(current, 'bytes 0-1023/${origin.objectLength}');
    expect(next, 'invalid');
  });
}

Future<String?> _range(HttpClient client, Uri url) async {
  final request = await client.getUrl(url);
  request.headers.set(HttpHeaders.rangeHeader, 'bytes=0-1023');
  final response = await request.close();
  await response.drain<void>();
  return response.headers.value(HttpHeaders.contentRangeHeader);
}
