import 'dart:io';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/warp_validator_rotation_fixture.dart';

void main() {
  test('stale If-Range receives the exact replacement generation', () async {
    final fixture = await WarpValidatorRotationFixture.start(
      holdFirstGeneration: false,
    );
    addTearDown(fixture.close);

    final first = await _get(fixture.mediaUrl, range: 'bytes=0-63');
    expect(first.status, HttpStatus.partialContent);
    expect(first.etag, fixture.firstValidator);
    expect(first.bytes, fixture.firstBytes.sublist(0, 64));
    expect(fixture.redirectTargets.single.path, '/generation-a.mp4');

    fixture.rotate();
    final changed = await _get(
      fixture.mediaUrl,
      range: 'bytes=64-127',
      ifRange: fixture.firstValidator,
    );
    expect(changed.status, HttpStatus.ok);
    expect(changed.etag, fixture.secondValidator);
    expect(changed.bytes, fixture.secondBytes);
    expect(changed.bytes.length, first.totalLength);
    expect(
      sha256.convert(changed.bytes),
      isNot(sha256.convert(fixture.firstBytes)),
    );
    expect(fixture.requests.last.range, 'bytes=64-127');
    expect(fixture.requests.last.ifRange, fixture.firstValidator);
    expect(fixture.redirectTargets.map((target) => target.path), [
      '/generation-a.mp4',
      '/generation-b.mp4',
    ]);
    expect(fixture.totalRequestCount, 4);
  });
}

Future<_Response> _get(
  Uri url, {
  required String range,
  String? ifRange,
}) async {
  final client = HttpClient();
  try {
    final request = await client.getUrl(url);
    request.headers.set(HttpHeaders.rangeHeader, range);
    if (ifRange != null) {
      request.headers.set(HttpHeaders.ifRangeHeader, ifRange);
    }
    final response = await request.close();
    final bytes = await response.fold<BytesBuilder>(
      BytesBuilder(copy: false),
      (builder, chunk) => builder..add(chunk),
    );
    return (
      status: response.statusCode,
      etag: response.headers.value(HttpHeaders.etagHeader),
      totalLength: fixtureLength(response, bytes.length),
      bytes: bytes.takeBytes(),
    );
  } finally {
    client.close(force: true);
  }
}

int fixtureLength(HttpClientResponse response, int received) {
  final contentRange = response.headers.value(HttpHeaders.contentRangeHeader);
  if (contentRange == null) return received;
  return int.parse(contentRange.substring(contentRange.lastIndexOf('/') + 1));
}

typedef _Response = ({
  int status,
  String? etag,
  int totalLength,
  Uint8List bytes,
});
