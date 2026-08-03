import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_inventory/domain/media_url_policy.dart';
import 'package:ghostr/platform/media/http_video_file_downloader.dart';
import 'package:ghostr/platform/media/video_download_timeouts.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

void main() {
  test('bounds stalled media address validation by the total deadline',
      () async {
    var sent = false;
    final client = MockClient((_) async {
      sent = true;
      return http.Response('', 200);
    });
    final downloader = HttpVideoFileDownloader(
      client,
      _StalledPolicy(),
      timeouts: const VideoDownloadTimeouts(
        headers: Duration(seconds: 1),
        idle: Duration(seconds: 1),
        total: Duration(milliseconds: 20),
      ),
    );

    await expectLater(
      downloader.download(
        Uri.parse('https://media.test/stalled.mp4'),
        '/unwritten/stalled.partial',
        maxBytes: 10,
      ),
      throwsA(isA<AppFailure>()),
    );
    expect(sent, isFalse);
  });
}

class _StalledPolicy implements MediaUrlPolicy {
  @override
  Future<void> validate(Uri source) => Completer<void>().future;
}
