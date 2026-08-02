import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/media/http_video_file_downloader.dart';
import 'package:http/testing.dart';

void main() {
  test('translates an HTTP transport failure into an app-safe failure',
      () async {
    final client = MockClient((_) => throw StateError('connection closed'));

    final future = HttpVideoFileDownloader(client).download(
      Uri.parse('https://media.test/video.mp4'),
      '/unwritten/video.partial',
      maxBytes: 10,
    );

    await expectLater(
      future,
      throwsA(isA<AppFailure>().having(
        (failure) => failure.message,
        'message',
        'The video could not be cached.',
      )),
    );
  });
}
