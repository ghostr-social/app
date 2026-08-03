import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/media/http_video_file_downloader.dart';
import 'package:ghostr/platform/media/video_download_timeouts.dart';
import 'package:http/http.dart' as http;

import '../support/allow_all_media_url_policy.dart';

void main() {
  test('uses the caller remaining timeout to abort an HTTP request', () async {
    final client = _StalledClient();
    final downloader = HttpVideoFileDownloader(
      client,
      const AllowAllMediaUrlPolicy(),
      timeouts: const VideoDownloadTimeouts(
        headers: Duration(seconds: 1),
        idle: Duration(seconds: 1),
        total: Duration(minutes: 1),
      ),
    );

    await expectLater(
      downloader.download(
        Uri.parse('https://media.test/stalled.mp4'),
        '/unwritten/stalled.partial',
        maxBytes: 10,
        totalTimeout: const Duration(milliseconds: 20),
      ),
      throwsA(isA<AppFailure>()),
    );
    await expectLater(client.aborted.future, completes);
  });
}

class _StalledClient extends http.BaseClient {
  final aborted = Completer<void>();

  @override
  Future<http.StreamedResponse> send(http.BaseRequest request) {
    final trigger = (request as http.AbortableRequest).abortTrigger!;
    final response = Completer<http.StreamedResponse>();
    trigger.then((_) {
      aborted.complete();
      response.completeError(StateError('aborted'));
    });
    return response.future;
  }
}
