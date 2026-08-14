import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/data/http_app_release_catalog.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

import '../support/update_manifest_fixture.dart';

void main() {
  test('falls back to Pages when the latest release is unavailable', () async {
    final latest = Uri.parse(
      'https://github.com/ghostr-social/app/releases/latest/download/stable.json',
    );
    final pages = Uri.parse('https://ghostr-social.github.io/stable.json');
    final requested = <Uri>[];
    final client = MockClient.streaming((request, body) async {
      requested.add(request.url);
      if (request.url == latest) {
        return http.StreamedResponse(const Stream.empty(), 503);
      }
      final bytes = utf8.encode(stableManifestJson());
      return http.StreamedResponse(
        Stream.value(bytes),
        HttpStatus.ok,
        contentLength: bytes.length,
      );
    });

    final release = await HttpAppReleaseCatalog(client).fetchStableRelease();

    expect(requested, [latest, pages]);
    expect(release.versionCode.value, 1002003);
  });
}
