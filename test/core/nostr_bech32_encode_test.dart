import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_bech32.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('encodes a public key to the npub its decoder accepts', () {
    final bytes = nostrKeyBytes(testViewerPublicKey);

    final npub = encodeNostrBech32Key('npub', bytes!);

    expect(npub, testViewerNpub);
    expect(
        nostrKeyHex(decodeNostrBech32Key(npub!, 'npub')!), testViewerPublicKey);
  });
}
