import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/data/http_app_release_catalog.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

import '../support/update_manifest_fixture.dart';

void main() {
  test(
    'allows composition to inject a future custom-domain endpoint',
    () async {
      final endpoint = Uri.parse('https://ghostr.social/updates/stable.json');
      late Uri requested;
      final client = MockClient.streaming((request, body) async {
        requested = request.url;
        final bytes = utf8.encode(stableManifestJson());
        return http.StreamedResponse(Stream.value(bytes), 200);
      });

      await HttpAppReleaseCatalog(
        client,
        endpoint: endpoint,
      ).fetchStableRelease();

      expect(requested, endpoint);
    },
  );
}
