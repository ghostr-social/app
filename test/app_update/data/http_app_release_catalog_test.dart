import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/data/http_app_release_catalog.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

import '../support/update_manifest_fixture.dart';

void main() {
  test(
    'fetches and parses bounded stable metadata from ghostr.social',
    () async {
      late http.BaseRequest captured;
      final client = MockClient.streaming((request, body) async {
        captured = request;
        final bytes = utf8.encode(stableManifestJson());
        return http.StreamedResponse(
          Stream.value(bytes),
          200,
          contentLength: bytes.length,
        );
      });

      final release = await HttpAppReleaseCatalog(client).fetchStableRelease();

      expect(captured.url, HttpAppReleaseCatalog.stableEndpoint);
      expect(
        captured.url.toString(),
        'https://ghostr-social.github.io/stable.json',
      );
      expect(captured.followRedirects, isFalse);
      expect(release.versionCode.value, 1002003);
    },
  );
}
