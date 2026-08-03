import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_bech32.dart';

void main() {
  test('rejects malformed key material instead of encoding it', () {
    expect(nostrKeyBytes('abc'), isNull);
    expect(nostrKeyBytes('zz' * 32), isNull);
    expect(encodeNostrBech32Key('npub', List<int>.filled(31, 0)), isNull);
    expect(encodeNostrBech32Key('npub', List<int>.filled(32, 256)), isNull);
  });
}
