import 'package:ghostr/core/nostr/nostr_event_identity.dart';

String? projectRelayProfileText(Object? raw, int maximumRunes) {
  final normalized = _normalizedText(raw);
  if (normalized == null) return null;
  final runes = normalized.runes;
  return runes.length <= maximumRunes
      ? normalized
      : String.fromCharCodes(runes.take(maximumRunes));
}

String? projectRelayProfileIdentityText(
  Object? raw,
  int maximumRunes,
  NostrPublicKeyHex author,
) {
  final value = _withoutRawAuthor(_normalizedText(raw), author);
  return projectRelayProfileText(value, maximumRunes);
}

String? projectRelayProfileIdentityHandle(
  Object? raw,
  NostrPublicKeyHex author,
) {
  final value = _withoutRawAuthor(_relayHandleText(raw), author);
  return projectRelayProfileText(value, 30);
}

String? projectRelayProfilePicture(Object? raw) {
  if (raw is! String || raw.length > 2048) return null;
  final value = raw.trim();
  final uri = Uri.tryParse(value);
  if (uri == null || uri.host.isEmpty || uri.userInfo.isNotEmpty) return null;
  return uri.scheme == 'https' || uri.scheme == 'http' ? uri.toString() : null;
}

String? _normalizedText(Object? raw) {
  if (raw is! String) return null;
  final normalized = raw.replaceAll(_unsafeText, ' ').trim();
  return normalized.isEmpty ? null : normalized;
}

String? _relayHandleText(Object? raw) {
  final normalized = _normalizedText(raw);
  if (normalized == null) return null;
  final value = normalized.replaceFirst(RegExp(r'^@+\s*'), '');
  return value.isEmpty ? null : value;
}

String? _withoutRawAuthor(String? value, NostrPublicKeyHex author) {
  if (value?.toLowerCase() == author.value) return null;
  return value;
}

final _unsafeText = RegExp(r'[\x00-\x20\x7F-\x9F\u202A-\u202E\u2066-\u2069]+');
