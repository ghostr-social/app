import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';
import '../../integration_test/support/progressive_mp4_fixture.dart';

void main() {
  test('one origin serves encrypted HLS and ranged MP4 rescue', () async {
    final origin = await ProgressiveDeviceOrigin.start();
    final client = HttpClient();
    addTearDown(() async {
      client.close(force: true);
      await origin.close();
    });

    final manifest = await (await client.getUrl(
      origin.encryptedHlsUrl,
    )).close();
    final manifestBody = await utf8.decodeStream(manifest);
    final rescueRequest = await client.getUrl(origin.urlFor('hls-rescue'));
    rescueRequest.headers.set(HttpHeaders.rangeHeader, 'bytes=0-63');
    final rescue = await rescueRequest.close();
    final rescueBody = await rescue.fold<List<int>>([], (all, bytes) {
      return all..addAll(bytes);
    });

    expect(origin.encryptedHlsUrl.origin, origin.urlFor('hls-rescue').origin);
    expect(manifest.statusCode, HttpStatus.ok);
    expect(manifestBody, startsWith('#EXTM3U\n'));
    expect(manifestBody, contains('#EXT-X-KEY:METHOD=AES-128'));
    expect(manifestBody, contains('#EXT-X-ENDLIST'));
    expect(rescue.statusCode, HttpStatus.partialContent);
    expect(rescueBody, ProgressiveMp4Fixture.bytes.sublist(0, 64));
    expect(origin.requests, hasLength(2));
    expect(origin.requests.first.path, '/encrypted/index.m3u8');
    expect(origin.requests.last.range, (start: 0, end: 64));
  });
}
