import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/app_update/data/http_app_release_catalog.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

void main() {
  test('uses catalog wording when a custom endpoint is rejected', () async {
    var calls = 0;
    final client = MockClient.streaming((request, body) async {
      calls += 1;
      return http.StreamedResponse(const Stream.empty(), 200);
    });
    final result = HttpAppReleaseCatalog(
      client,
      endpoint: Uri.parse('http://example.com/stable.json'),
    ).fetchStableRelease();

    await expectLater(
      result,
      throwsA(
        isA<AppFailure>().having(
          (failure) => failure.message,
          'message',
          'Could not check for updates.',
        ),
      ),
    );
    expect(calls, 0);
  });
}
