import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/data/http_app_release_catalog.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

import '../support/update_manifest_fixture.dart';

void main() {
  test('follows GitHub redirects to the latest stable manifest', () async {
    final latest = Uri.parse(
      'https://github.com/ghostr-social/app/releases/latest/download/stable.json',
    );
    final tagged = Uri.parse(
      'https://github.com/ghostr-social/app/releases/download/v1.2.3/stable.json',
    );
    final asset = Uri.parse(
      'https://release-assets.githubusercontent.com/stable.json',
    );
    final requested = <Uri>[];
    final client = MockClient.streaming((request, body) async {
      requested.add(request.url);
      if (request.url == latest) return _redirect(tagged);
      if (request.url == tagged) return _redirect(asset);
      final bytes = utf8.encode(stableManifestJson());
      return http.StreamedResponse(
        Stream.value(bytes),
        HttpStatus.ok,
        contentLength: bytes.length,
      );
    });

    final release = await HttpAppReleaseCatalog(client).fetchStableRelease();

    expect(requested, [latest, tagged, asset]);
    expect(release.versionCode.value, 1002003);
  });
}

http.StreamedResponse _redirect(Uri target) => http.StreamedResponse(
  const Stream.empty(),
  HttpStatus.found,
  headers: {HttpHeaders.locationHeader: target.toString()},
);
