import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';

void main() {
  test('rejects relay URLs with credentials, queries, or fragments', () {
    expect(RelayUrl.tryParse('wss://user@relay.example'), isNull);
    expect(RelayUrl.tryParse('wss://relay.example?token=secret'), isNull);
    expect(RelayUrl.tryParse('wss://relay.example#fragment'), isNull);
  });
}
