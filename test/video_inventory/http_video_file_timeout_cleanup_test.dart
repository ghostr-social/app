import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/media/http_video_file_downloader.dart';
import 'package:ghostr/platform/media/video_download_timeouts.dart';
import 'package:http/http.dart' as http;

import '../support/allow_all_media_url_policy.dart';

void main() {
  test('waits for aborted response and sink cleanup before completing',
      () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-http-');
    addTearDown(() => directory.delete(recursive: true));
    final destination = File('${directory.path}/video.partial');
    final client = _ControlledAbortClient();
    addTearDown(client.releaseCleanup);
    final downloader = HttpVideoFileDownloader(
      client,
      const AllowAllMediaUrlPolicy(),
      timeouts: const VideoDownloadTimeouts(
        headers: Duration(seconds: 1),
        idle: Duration(seconds: 1),
        total: Duration(minutes: 1),
      ),
    );
    var completed = false;

    final download = downloader
        .download(
          Uri.parse('https://media.test/video.mp4'),
          destination.path,
          maxBytes: 10,
          totalTimeout: const Duration(milliseconds: 20),
        )
        .whenComplete(() => completed = true);
    final completionExpectation = expectLater(
      download,
      throwsA(isA<AppFailure>()),
    );
    await client.abortReceived.future;
    await Future<void>.delayed(Duration.zero);

    expect(completed, isFalse);
    client.releaseCleanup();
    await completionExpectation;
    expect(client.cleanupFinished.isCompleted, isTrue);
    await destination.delete();
    await Future<void>.delayed(const Duration(milliseconds: 20));
    expect(await destination.exists(), isFalse);
  });
}

class _ControlledAbortClient extends http.BaseClient {
  final abortReceived = Completer<void>();
  final cleanupFinished = Completer<void>();
  final _release = Completer<void>();
  final _body = StreamController<List<int>>();

  @override
  Future<http.StreamedResponse> send(http.BaseRequest request) async {
    final trigger = (request as http.AbortableRequest).abortTrigger!;
    trigger.then((_) async {
      abortReceived.complete();
      await _release.future;
      _body.add(const [2]);
      await _body.close();
      cleanupFinished.complete();
    });
    _body.onListen = () => _body.add(const [1]);
    return http.StreamedResponse(_body.stream, HttpStatus.ok);
  }

  void releaseCleanup() {
    if (!_release.isCompleted) _release.complete();
  }
}
