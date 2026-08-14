import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/data/https_update_response_loader.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

void main() {
  test('times out while discarding a stalled redirect body', () async {
    var calls = 0;
    final client = MockClient.streaming((request, body) async {
      calls += 1;
      if (calls == 1) {
        final delayed = Future<void>.delayed(const Duration(milliseconds: 50));
        return http.StreamedResponse(
          Stream.fromFuture(delayed).map((_) => <int>[]),
          HttpStatus.found,
          headers: {'location': 'https://github.com/next'},
        );
      }
      return http.StreamedResponse(const Stream.empty(), HttpStatus.ok);
    });

    final result = HttpsUpdateResponseLoader(
      client,
      timeout: const Duration(milliseconds: 1),
    ).load(Uri.parse('https://github.com/start'));

    await expectLater(result, throwsA(isA<TimeoutException>()));
    expect(calls, 1);
  });
}
