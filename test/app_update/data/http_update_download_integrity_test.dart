import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:http/testing.dart';

import '../support/update_download_fixture.dart';

void main() {
  test(
    'rejects wrong response size and digest without retaining partials',
    () async {
      final fixture = UpdateDownloadFixture.create();
      addTearDown(fixture.dispose);
      final responses = [
        streamedResponse(fixture.chunks(), contentLength: 3),
        streamedResponse(Stream.value(const [1, 2, 3]), contentLength: 3),
        streamedResponse(Stream.value(const [1, 2, 3, 5]), contentLength: 4),
        streamedResponse(Stream.value(const [1, 2, 3, 4, 5])),
      ];

      for (final response in responses) {
        final downloader = fixture.downloader(
          MockClient.streaming((request, body) async => response),
        );
        await expectLater(
          downloader.download(fixture.release, fixture.artifact).toList(),
          throwsA(isA<AppFailure>()),
        );
        expect(await File(fixture.destination).exists(), isFalse);
        expect(await File(fixture.partial).exists(), isFalse);
      }
    },
  );
}
