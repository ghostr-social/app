import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_bech32.dart';
import 'package:ghostr/features/session/data/ndk_nostr_account_generator.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('generates a matching valid nsec and npub through injected steps', () {
    final privateKey = nostrKeyHex(decodeNostrBech32Key(testNsec, 'nsec')!);
    final calls = <String>[];
    final generator = NdkNostrAccountGenerator(
      steps: NostrAccountGenerationSteps(
        generatePrivateKey: () => privateKey,
        derivePublicKey: (value) {
          calls.add('derive:$value');
          return testViewerPublicKey;
        },
        encodeNsec: (value) {
          calls.add('nsec:$value');
          return testNsec;
        },
        encodeNpub: (value) {
          calls.add('npub:$value');
          return testViewerNpub;
        },
      ),
    );

    final account = generator.generate();

    expect(account.secret.value, testNsec);
    expect(account.identity.publicKeyHex.value, testViewerPublicKey);
    expect(account.identity.npub.value, testViewerNpub);
    expect(calls, [
      'derive:$privateKey',
      'nsec:$privateKey',
      'npub:$testViewerPublicKey',
    ]);
  });
}
