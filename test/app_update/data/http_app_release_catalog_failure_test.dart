import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/app_update/data/http_app_release_catalog.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

void main() {
  test(
    'translates rejected, malformed, and oversized catalog responses',
    () async {
      final responses = <http.StreamedResponse>[
        http.StreamedResponse(const Stream.empty(), 503),
        http.StreamedResponse(Stream.value(utf8.encode('{}')), 200),
        http.StreamedResponse(
          Stream.value(List<int>.filled(65537, 0)),
          200,
          contentLength: 65537,
        ),
      ];

      for (final response in responses) {
        final catalog = HttpAppReleaseCatalog(
          MockClient.streaming((request, body) async => response),
        );
        await expectLater(
          catalog.fetchStableRelease(),
          throwsA(isA<AppFailure>()),
        );
      }
    },
  );
}
