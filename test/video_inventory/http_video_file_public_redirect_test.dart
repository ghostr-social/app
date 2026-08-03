import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/http_video_file_downloader.dart';
import 'package:ghostr/platform/network/public_media_address_resolver.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

void main() {
  test('follows a relative redirect that resolves to a public URL', () async {
    final attempted = <Uri>[];
    final client = MockClient((request) async {
      attempted.add(request.url);
      if (attempted.length == 1) {
        return http.Response('', 302,
            headers: {'location': '../cdn/final.mp4'});
      }
      return http.Response.bytes([1, 2, 3], 200);
    });
    final policy = PublicMediaAddressResolver(
      lookup: (_) async => [InternetAddress('93.184.216.34')],
    );
    final directory = await Directory.systemTemp.createTemp('redirect-');
    addTearDown(() => directory.delete(recursive: true));
    final destination = File('${directory.path}/video.partial');

    await HttpVideoFileDownloader(client, policy).download(
      Uri.parse('https://media.test/videos/source.mp4'),
      destination.path,
      maxBytes: 3,
    );

    expect(attempted, [
      Uri.parse('https://media.test/videos/source.mp4'),
      Uri.parse('https://media.test/cdn/final.mp4'),
    ]);
    expect(await destination.readAsBytes(), [1, 2, 3]);
  });
}
