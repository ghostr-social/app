import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

import '../support/update_download_fixture.dart';

void main() {
  test(
    'follows bounded HTTPS redirects and rejects insecure locations',
    () async {
      final fixture = UpdateDownloadFixture.create();
      addTearDown(fixture.dispose);
      var calls = 0;
      final secure = MockClient.streaming((request, body) async {
        calls += 1;
        if (calls == 1) {
          return http.StreamedResponse(
            const Stream.empty(),
            HttpStatus.found,
            headers: {
              'location': 'https://release-assets.githubusercontent.com/a',
            },
          );
        }
        return streamedResponse(fixture.chunks(), contentLength: 4);
      });
      await fixture
          .downloader(secure)
          .download(fixture.release, fixture.artifact)
          .drain<void>();
      expect(calls, 2);
      await File(fixture.destination).delete();

      final insecure = MockClient.streaming((request, body) async {
        return http.StreamedResponse(
          const Stream.empty(),
          HttpStatus.found,
          headers: {'location': 'http://example.com/update.apk'},
        );
      });
      await expectLater(
        fixture
            .downloader(insecure)
            .download(fixture.release, fixture.artifact)
            .toList(),
        throwsA(isA<AppFailure>()),
      );

      final missingLocation = MockClient.streaming((request, body) async {
        return http.StreamedResponse(const Stream.empty(), HttpStatus.found);
      });
      await expectLater(
        fixture
            .downloader(missingLocation)
            .download(fixture.release, fixture.artifact)
            .toList(),
        throwsA(isA<AppFailure>()),
      );

      var redirectCalls = 0;
      final redirectLoop = MockClient.streaming((request, body) async {
        redirectCalls += 1;
        return http.StreamedResponse(
          const Stream.empty(),
          HttpStatus.found,
          headers: {'location': 'https://github.com/next'},
        );
      });
      await expectLater(
        fixture
            .downloader(redirectLoop)
            .download(fixture.release, fixture.artifact)
            .toList(),
        throwsA(isA<AppFailure>()),
      );
      expect(redirectCalls, 6);
    },
  );
}
