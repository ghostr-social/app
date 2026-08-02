import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/http_video_file_downloader.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

void main() {
  test('streams a successful HTTP response into the destination file',
      () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-http-');
    addTearDown(() => directory.delete(recursive: true));
    final destination = File('${directory.path}/video.partial');
    final client = MockClient((_) async => http.Response.bytes([1, 2, 3], 200));
    final downloader = HttpVideoFileDownloader(client);

    await downloader.download(
      Uri.parse('https://media.test/video.mp4'),
      destination.path,
      maxBytes: 3,
    );

    expect(await destination.readAsBytes(), [1, 2, 3]);
  });
}
