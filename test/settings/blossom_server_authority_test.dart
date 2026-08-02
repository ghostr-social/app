import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/blossom_server_url.dart';

void main() {
  test('rejects Blossom URLs with credentials, queries, or fragments', () {
    expect(BlossomServerUrl.tryParse('https://user@media.example'), isNull);
    expect(
        BlossomServerUrl.tryParse('https://media.example?key=value'), isNull);
    expect(BlossomServerUrl.tryParse('https://media.example#fragment'), isNull);
  });
}
