import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/media/http_video_file_downloader.dart';
import 'package:http/http.dart' as http;

import '../support/allow_all_media_url_policy.dart';

void main() {
  test('rejects a failed response without consuming its body', () async {
    final destination = await _destination();
    final downloader = HttpVideoFileDownloader(
      _ResponseClient(<http.StreamedResponse>[_forbiddenBody(503)]),
      const AllowAllMediaUrlPolicy(),
    );

    await expectLater(
      downloader.download(_source, destination.path, maxBytes: 10),
      throwsA(isA<AppFailure>().having(
        (failure) => failure.message,
        'message',
        contains('503'),
      )),
    );
  });

  test('follows a redirect without consuming its response body', () async {
    final destination = await _destination();
    final client = _ResponseClient(<http.StreamedResponse>[
      _forbiddenBody(302, headers: const {'location': '/final.mp4'}),
      http.StreamedResponse(Stream.value(<int>[1, 2, 3]), 200),
    ]);

    await HttpVideoFileDownloader(
      client,
      const AllowAllMediaUrlPolicy(),
    ).download(_source, destination.path, maxBytes: 10);

    expect(await destination.readAsBytes(), <int>[1, 2, 3]);
    expect(client.requested.last.path, '/final.mp4');
  });
}

final Uri _source = Uri.parse('https://media.test/video.mp4');

Future<File> _destination() async {
  final directory = await Directory.systemTemp.createTemp('ghostr-http-');
  addTearDown(() => directory.delete(recursive: true));
  return File('${directory.path}/video.partial');
}

http.StreamedResponse _forbiddenBody(
  int statusCode, {
  Map<String, String> headers = const <String, String>{},
}) {
  return http.StreamedResponse(
    Stream<List<int>>.error(StateError('response body was consumed')),
    statusCode,
    headers: headers,
  );
}

class _ResponseClient extends http.BaseClient {
  _ResponseClient(this._responses);

  final List<http.StreamedResponse> _responses;
  final List<Uri> requested = <Uri>[];

  @override
  Future<http.StreamedResponse> send(http.BaseRequest request) async {
    requested.add(request.url);
    return _responses.removeAt(0);
  }
}
