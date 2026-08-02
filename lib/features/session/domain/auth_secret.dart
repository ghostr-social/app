import 'package:ghostr/core/nostr/nostr_bech32.dart';

class AuthSecret {
  AuthSecret._(this.value);

  factory AuthSecret.parse(String raw) {
    final parsed = tryParse(raw);
    if (parsed == null) {
      throw const FormatException('Enter a valid nsec1 secret.');
    }
    return parsed;
  }

  static AuthSecret? tryParse(String raw) {
    final value = raw.trim();
    return decodeNostrBech32Key(value, 'nsec') == null
        ? null
        : AuthSecret._(value);
  }

  final String value;
}
