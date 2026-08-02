extension type const NostrEventId._(String value) implements String {
  factory NostrEventId.parse(String raw) {
    return NostrEventId._(_hex32(raw, 'Nostr event ID'));
  }
}

extension type const NostrPublicKeyHex._(String value) implements String {
  factory NostrPublicKeyHex.parse(String raw) {
    return NostrPublicKeyHex._(_hex32(raw, 'Nostr public key'));
  }
}

extension type const NostrEventKind._(int value) implements int {
  factory NostrEventKind.parse(int raw) {
    if (raw < 0 || raw > 65535) {
      throw const FormatException('Invalid Nostr event kind.');
    }
    return NostrEventKind._(raw);
  }
}

extension type const NostrEventIdentifier._(String value) implements String {
  factory NostrEventIdentifier.parse(String raw) {
    return NostrEventIdentifier._(_requiredText(raw, 'Nostr identifier'));
  }
}

class NostrEventIdentity {
  factory NostrEventIdentity.parse({
    required String id,
    required String authorPublicKeyHex,
    required int kind,
  }) {
    return NostrEventIdentity._(
      NostrEventId.parse(id),
      NostrPublicKeyHex.parse(authorPublicKeyHex),
      NostrEventKind.parse(kind),
    );
  }

  const NostrEventIdentity._(
    this.id,
    this.authorPublicKeyHex,
    this.kind,
  );

  final NostrEventId id;
  final NostrPublicKeyHex authorPublicKeyHex;
  final NostrEventKind kind;
}

String _requiredText(String raw, String label) {
  final value = raw.trim();
  if (value.isEmpty) throw FormatException('$label cannot be empty.');
  return value;
}

String _hex32(String raw, String label) {
  final value = raw.trim().toLowerCase();
  if (!RegExp(r'^[0-9a-f]{64}$').hasMatch(value)) {
    throw FormatException('$label must be a 32-byte hexadecimal value.');
  }
  return value;
}
