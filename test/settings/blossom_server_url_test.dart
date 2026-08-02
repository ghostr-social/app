import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/blossom_server_url.dart';

void main() {
  test('accepts secure Blossom servers and local development HTTP', () {
    expect(
      BlossomServerUrl.parse('https://blossom.primal.net/').value,
      'https://blossom.primal.net',
    );
    expect(
      BlossomServerUrl.parse('http://localhost:3000').value,
      'http://localhost:3000',
    );
    expect(
      () => BlossomServerUrl.parse('http://media.example'),
      throwsFormatException,
    );
  });
}
