import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/media/http_video_file_downloader.dart';
import 'package:ghostr/platform/media/video_download_timeouts.dart';
import 'package:http/http.dart' as http;

import '../support/allow_all_media_url_policy.dart';

void main() {
  test('aborts a download whose response body stalls', () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-stall-');
    addTearDown(() => directory.delete(recursive: true));
    final client = _StalledBodyClient();
    addTearDown(client.close);
    final downloader = HttpVideoFileDownloader(
      client,
      const AllowAllMediaUrlPolicy(),
      timeouts: const VideoDownloadTimeouts(
        headers: Duration(seconds: 1),
        idle: Duration(milliseconds: 20),
        total: Duration(seconds: 1),
      ),
    );

    await expectLater(
      downloader.download(
        Uri.parse('https://media.test/stalled.mp4'),
        '${directory.path}/video.partial',
        maxBytes: 10,
      ),
      throwsA(isA<AppFailure>()),
    );
    await expectLater(client.aborted.future, completes);
  });
}

class _StalledBodyClient extends http.BaseClient {
  final aborted = Completer<void>();
  final body = StreamController<List<int>>();

  @override
  Future<http.StreamedResponse> send(http.BaseRequest request) async {
    final trigger = (request as http.AbortableRequest).abortTrigger!;
    trigger.then((_) async {
      aborted.complete();
      await body.close();
    });
    body.onListen = () => body.add([1]);
    return http.StreamedResponse(body.stream, 200);
  }

  @override
  void close() {
    if (!body.isClosed) body.close();
  }
}
