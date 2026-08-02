import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';

void main() {
  test('requires encrypted remote relays while allowing local development', () {
    expect(
      RelayUrl.tryParse('wss://relay.example/')?.value,
      'wss://relay.example',
    );
    expect(RelayUrl.tryParse('ws://localhost:7447'), isNotNull);
    expect(RelayUrl.tryParse('ws://127.0.0.1:7447'), isNotNull);
    expect(RelayUrl.tryParse('ws://relay.example'), isNull);
    expect(RelayUrl.tryParse('https://relay.example'), isNull);
  });
}
