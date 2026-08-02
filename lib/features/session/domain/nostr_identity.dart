import 'package:ghostr/core/nostr/nostr_bech32.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';

extension type const NostrNpub._(String value) implements String {
  factory NostrNpub.parse(String raw) {
    final value = raw.trim();
    if (decodeNostrBech32Key(value, 'npub') == null) {
      throw const FormatException('Invalid Nostr npub.');
    }
    return NostrNpub._(value);
  }

  String get publicKeyHex {
    return nostrKeyHex(decodeNostrBech32Key(value, 'npub')!);
  }
}

class NostrIdentity {
  factory NostrIdentity.parse({
    required String publicKeyHex,
    required String npub,
  }) {
    final key = NostrPublicKeyHex.parse(publicKeyHex);
    final encoded = NostrNpub.parse(npub);
    if (encoded.publicKeyHex != key) {
      throw const FormatException('Nostr identity keys do not match.');
    }
    return NostrIdentity._(key, encoded);
  }

  const NostrIdentity._(this.publicKeyHex, this.npub);

  final NostrPublicKeyHex publicKeyHex;
  final NostrNpub npub;
}
