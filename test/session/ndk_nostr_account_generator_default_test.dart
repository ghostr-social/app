import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_bech32.dart';
import 'package:ghostr/features/session/data/ndk_nostr_account_generator.dart';
import 'package:ndk/ndk.dart';

void main() {
  test('default NDK generation creates a matching Nostr keypair', () {
    final account = const NdkNostrAccountGenerator().generate();
    final privateBytes = decodeNostrBech32Key(account.secret.value, 'nsec');
    final privateKey = nostrKeyHex(privateBytes!);

    final derived = const Bip340EventSignerFactory().derivePublicKey(
      privateKey,
    );

    expect(derived, account.identity.publicKeyHex.value);
    expect(account.identity.npub.publicKeyHex, account.identity.publicKeyHex);
  });
}
