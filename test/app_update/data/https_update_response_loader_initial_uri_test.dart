import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/app_update/data/https_update_response_loader.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

void main() {
  test('rejects an insecure initial URI before sending a request', () async {
    var calls = 0;
    final client = MockClient.streaming((request, body) async {
      calls += 1;
      return http.StreamedResponse(const Stream.empty(), 200);
    });

    final result = HttpsUpdateResponseLoader(
      client,
    ).load(Uri.parse('http://example.com/stable.json'));

    await expectLater(result, throwsA(isA<AppFailure>()));
    expect(calls, 0);
  });
}
