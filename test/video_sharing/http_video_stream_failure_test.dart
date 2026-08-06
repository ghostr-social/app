import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/sharing/http_video_file_transfer.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

void main() {
  test('cleans a partial file when gateway streaming fails', () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-share');
    final destination = '${directory.path}/clip.mp4';
    final transfer = HttpVideoFileTransfer(
      MockClient.streaming((request, body) {
        return Future.value(
          http.StreamedResponse(
            Stream<List<int>>.error(StateError('stream failed')),
            HttpStatus.ok,
            contentLength: 3,
          ),
        );
      }),
    );

    await expectLater(
      transfer.transfer(
        Uri.parse('http://127.0.0.1:1234/video.mp4?id=clip'),
        destination,
      ),
      throwsA(isA<AppFailure>()),
    );
    expect(File('$destination.partial').existsSync(), isFalse);
    await directory.delete(recursive: true);
  });
}
