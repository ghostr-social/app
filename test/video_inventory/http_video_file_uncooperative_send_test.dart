import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/media/http_video_file_downloader.dart';
import 'package:ghostr/platform/media/video_download_timeouts.dart';
import 'package:http/http.dart' as http;

import '../support/allow_all_media_url_policy.dart';

void main() {
  test('does not await an uncooperative request after header abort', () async {
    final client = _UncooperativeClient();
    addTearDown(client.releaseWithError);
    final downloader = HttpVideoFileDownloader(
      client,
      const AllowAllMediaUrlPolicy(),
      timeouts: const VideoDownloadTimeouts(
        headers: Duration(milliseconds: 20),
        idle: Duration(seconds: 1),
        total: Duration(seconds: 1),
      ),
    );
    var completed = false;
    final download = downloader
        .download(
          Uri.parse('https://media.test/video.mp4'),
          '/unwritten/video.partial',
          maxBytes: 10,
        )
        .whenComplete(() => completed = true);
    final completionExpectation = expectLater(
      download,
      throwsA(isA<AppFailure>()),
    );

    await client.abortReceived.future;
    await Future<void>.delayed(Duration.zero);

    final completedBeforeRelease = completed;
    client.releaseWithError();
    await completionExpectation;
    await Future<void>.delayed(Duration.zero);
    expect(completedBeforeRelease, isTrue);
  });
}

class _UncooperativeClient extends http.BaseClient {
  final abortReceived = Completer<void>();
  final _response = Completer<http.StreamedResponse>();

  @override
  Future<http.StreamedResponse> send(http.BaseRequest request) {
    final trigger = (request as http.AbortableRequest).abortTrigger!;
    trigger.then((_) => abortReceived.complete());
    return _response.future;
  }

  void releaseWithError() {
    if (!_response.isCompleted) {
      _response.completeError(StateError('late transport failure'));
    }
  }
}
