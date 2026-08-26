import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test('device origin serves a range while its HEAD remains blocked', () async {
    final origin = await ProgressiveDeviceOrigin.start(
      validator: ProgressiveOriginValidator.stableStrong,
    );
    final client = HttpClient();
    addTearDown(() async {
      client.close(force: true);
      await origin.close();
    });
    final head = _head(client, origin.urlFor('current'));
    await Future<void>.delayed(const Duration(milliseconds: 20));

    final request = await client.getUrl(origin.urlFor('current'));
    request.headers.set(HttpHeaders.rangeHeader, 'bytes=0-2047');
    final response = await request.close();
    final body = await response.fold<List<int>>(
      [],
      (all, part) => all..addAll(part),
    );

    expect(response.statusCode, HttpStatus.partialContent);
    expect(response.headers.value(HttpHeaders.etagHeader), '"warp-fixture-v1"');
    expect(body, hasLength(2048));
    await expectLater(
      head.timeout(const Duration(milliseconds: 20)),
      throwsA(isA<TimeoutException>()),
    );
    expect(origin.requests.map((request) => request.method), ['HEAD', 'GET']);
    expect(origin.requests.map((request) => request.outcome), [
      ProgressiveOriginRequestOutcome.headBlocked,
      ProgressiveOriginRequestOutcome.completed,
    ]);
    expect(origin.requests.last.servedBytes, 2048);
  });
}

Future<void> _head(HttpClient client, Uri uri) async {
  final request = await client.openUrl('HEAD', uri);
  await request.close();
}
