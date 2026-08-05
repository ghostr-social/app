import 'dart:convert';

import 'package:ghostr/features/social/domain/signed_nostr_event_json.dart';
import 'package:ndk/ndk.dart';

/// The canonical NIP-01 wire form of a signed event, the only payload a
/// [SignedEventBroadcastPort] carries. Relay metadata that ndk keeps on
/// the event (sources, signature verdicts) is not part of the wire form.
SignedNostrEventJson encodeSignedNostrEvent(Nip01Event event) {
  final json = jsonEncode(_signedNostrEventPayload(event));
  return SignedNostrEventJson.parse(json);
}

/// The NIP-01 fields of [event], in the order the spec lists them.
Map<String, Object?> _signedNostrEventPayload(Nip01Event event) {
  final signature = event.sig;
  if (signature == null || signature.isEmpty) {
    throw const FormatException('A Nostr event must be signed to broadcast.');
  }
  return <String, Object?>{
    'id': event.id,
    'pubkey': event.pubKey,
    'created_at': event.createdAt,
    'kind': event.kind,
    'tags': event.tags,
    'content': event.content,
    'sig': signature,
  };
}

/// Reads a signed event back, keeping the id and signature that were
/// written rather than recomputing either.
Nip01Event decodeSignedNostrEvent(SignedNostrEventJson signedEventJson) {
  final payload = jsonDecode(signedEventJson.value);
  if (payload is! Map<String, Object?>) {
    throw const FormatException('A signed Nostr event must be an object.');
  }
  return Nip01Event(
    id: _text(payload, 'id'),
    pubKey: _text(payload, 'pubkey'),
    createdAt: _whole(payload, 'created_at'),
    kind: _whole(payload, 'kind'),
    tags: _tags(payload['tags']),
    content: _text(payload, 'content'),
    sig: _text(payload, 'sig'),
  );
}

String _text(Map<String, Object?> payload, String field) {
  final value = payload[field];
  if (value is! String) {
    throw FormatException('Nostr event field "$field" must be text.');
  }
  return value;
}

int _whole(Map<String, Object?> payload, String field) {
  final value = payload[field];
  if (value is! int) {
    throw FormatException('Nostr event field "$field" must be a number.');
  }
  return value;
}

List<List<String>> _tags(Object? value) {
  if (value is! List) {
    throw const FormatException('Nostr event tags must be a list.');
  }
  return value.map(_tag).toList();
}

List<String> _tag(Object? value) {
  if (value is! List) {
    throw const FormatException('Every Nostr event tag must be a list.');
  }
  return value.map(_tagEntry).toList();
}

String _tagEntry(Object? value) {
  if (value is! String) {
    throw const FormatException('Nostr tag entries must be text.');
  }
  return value;
}
