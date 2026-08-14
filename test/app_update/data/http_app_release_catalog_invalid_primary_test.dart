import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/app_update/data/http_app_release_catalog.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

void main() {
  test(
    'does not hide an invalid canonical manifest with stale fallback',
    () async {
      final requested = <Uri>[];
      final bytes = utf8.encode('{}');
      final client = MockClient.streaming((request, body) async {
        requested.add(request.url);
        return http.StreamedResponse(
          Stream.value(bytes),
          HttpStatus.ok,
          contentLength: bytes.length,
        );
      });

      final result = HttpAppReleaseCatalog(client).fetchStableRelease();

      await expectLater(result, throwsA(isA<AppFailure>()));
      expect(requested, [HttpAppReleaseCatalog.stableEndpoint]);
    },
  );
}
