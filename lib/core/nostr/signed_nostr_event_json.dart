import 'dart:convert';

/// Validated canonical NIP-01 JSON for an event that already has a signature.
final class SignedNostrEventJson {
  factory SignedNostrEventJson.parse(String raw) {
    final payload = _eventPayload(jsonDecode(raw));
    _validateEventPayload(payload);
    return SignedNostrEventJson._(raw, _isProtected(payload));
  }

  const SignedNostrEventJson._(this.value, this.isProtected);

  final String value;
  final bool isProtected;

  @override
  bool operator ==(Object other) {
    return other is SignedNostrEventJson && other.value == value;
  }

  @override
  int get hashCode => value.hashCode;
}

Map<String, Object?> _eventPayload(Object? decoded) {
  if (decoded is! Map<String, Object?>) {
    throw const FormatException('A signed Nostr event must be an object.');
  }
  return decoded;
}

void _validateEventPayload(Map<String, Object?> payload) {
  _requiredText(payload, 'id');
  _requiredText(payload, 'pubkey');
  _wholeNumber(payload, 'created_at');
  _wholeNumber(payload, 'kind');
  _validateTags(payload['tags']);
  _text(payload, 'content');
  _requiredText(payload, 'sig');
}

bool _isProtected(Map<String, Object?> payload) {
  return (payload['tags'] as List).any((tag) {
    return tag is List && tag.length == 1 && tag.single == '-';
  });
}

String _requiredText(Map<String, Object?> payload, String field) {
  final value = _text(payload, field);
  if (value.isEmpty) {
    throw FormatException('Nostr event field "$field" cannot be empty.');
  }
  return value;
}

String _text(Map<String, Object?> payload, String field) {
  final value = payload[field];
  if (value is! String) {
    throw FormatException('Nostr event field "$field" must be text.');
  }
  return value;
}

void _wholeNumber(Map<String, Object?> payload, String field) {
  if (payload[field] is! int) {
    throw FormatException('Nostr event field "$field" must be a number.');
  }
}

void _validateTags(Object? value) {
  if (value is! List) {
    throw const FormatException('Nostr event tags must be a list.');
  }
  for (final tag in value) {
    if (tag is! List || tag.any((entry) => entry is! String)) {
      throw const FormatException('Nostr event tags must contain text lists.');
    }
  }
}
