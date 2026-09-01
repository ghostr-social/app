import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test(
    'HLS origin distinguishes a selected child from its alternate',
    () async {
      final origin = await ProgressiveDeviceOrigin.start();
      final client = HttpClient();
      addTearDown(() async {
        client.close(force: true);
        await origin.close();
      });
      final masterUri = origin.hlsUrlFor('multivariant');

      final master = await _get(client, masterUri);
      final selected = await _get(client, masterUri.resolve('selected.m3u8'));
      final alternate = await _get(client, masterUri.resolve('alternate.m3u8'));

      expect(master.status, HttpStatus.ok);
      expect(master.body, contains('#EXT-X-STREAM-INF:BANDWIDTH=1000000'));
      expect(master.body, contains('selected.m3u8'));
      expect(master.body, contains('alternate.m3u8'));
      expect(selected.status, HttpStatus.ok);
      expect(selected.body, contains('#EXT-X-MAP:URI="init.mp4"'));
      expect(selected.body, contains('#EXTINF:1.000000,'));
      expect(alternate.status, HttpStatus.ok);
      expect(origin.hlsRequestsFor('index.m3u8'), 1);
      expect(origin.hlsRequestsFor('selected.m3u8'), 1);
      expect(origin.hlsRequestsFor('alternate.m3u8'), 1);
    },
  );
}

Future<({int status, String body})> _get(HttpClient client, Uri uri) async {
  final response = await (await client.getUrl(uri)).close();
  return (status: response.statusCode, body: await utf8.decodeStream(response));
}
