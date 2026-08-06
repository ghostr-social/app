import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/sharing/http_video_file_transfer.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

void main() {
  test('streams successful gateway bytes into the destination file', () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-share');
    final destination = '${directory.path}/clip.mp4';
    await File(destination).writeAsBytes(<int>[9]);
    final transfer = HttpVideoFileTransfer(
      MockClient.streaming((request, body) {
        return Future.value(
          http.StreamedResponse(
            Stream<List<int>>.value(<int>[1, 2, 3]),
            HttpStatus.ok,
            contentLength: 3,
          ),
        );
      }),
    );

    await transfer.transfer(
      Uri.parse('http://127.0.0.1:1234/video.mp4?id=clip'),
      destination,
    );

    expect(await File(destination).readAsBytes(), <int>[1, 2, 3]);
    await directory.delete(recursive: true);
  });
}
