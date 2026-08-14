import 'package:ghostr/core/network/secure_endpoint_policy.dart';
import 'package:ghostr/core/nostr/nostr_event_client.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';

const _relayListKind = 10002;
const _relayListLimit = 32;
const _privateSuffixes = <String>['.internal', '.lan', '.local', '.localhost'];
const _endpointPolicy = SecureEndpointPolicy(
  secureScheme: 'wss',
  localDevelopmentScheme: 'ws',
);

final class NostrAuthorWriteRelayLookup {
  const NostrAuthorWriteRelayLookup(this._client);

  final NostrEventClient _client;

  Future<String?> call(NostrPublicKeyHex author) async {
    final events = await _client.query(_relayListQuery(author));
    final latest = _latest(events);
    return latest == null ? null : _firstWriteRelay(latest.tags);
  }
}

NostrEventQuery _relayListQuery(NostrPublicKeyHex author) {
  return NostrEventQuery(
    kinds: const [_relayListKind],
    scope: NostrEventQueryScope(authors: [author]),
    limit: _relayListLimit,
  );
}

NostrEventRecord? _latest(List<NostrEventRecord> events) {
  NostrEventRecord? latest;
  for (final event in events) {
    if (latest == null || _isNewer(event, latest)) latest = event;
  }
  return latest;
}

bool _isNewer(NostrEventRecord candidate, NostrEventRecord current) {
  if (candidate.createdAt != current.createdAt) {
    return candidate.createdAt > current.createdAt;
  }
  return candidate.id.value.compareTo(current.id.value) < 0;
}

String? _firstWriteRelay(Iterable<List<String>> tags) {
  for (final tag in tags) {
    final relay = _writableRelay(tag);
    if (relay != null) return relay;
  }
  return null;
}

String? _writableRelay(List<String> tag) {
  if (!_isRelayTag(tag)) return null;
  final marker = _relayMarker(tag);
  if (!_isWritableMarker(marker)) return null;
  return _publicRelay(tag[1]);
}

bool _isRelayTag(List<String> tag) => tag.length >= 2 && tag.first == 'r';

String? _relayMarker(List<String> tag) => tag.length > 2 ? tag[2] : null;

bool _isWritableMarker(String? marker) => marker == null || marker == 'write';

String? _publicRelay(String raw) {
  final normalized = _endpointPolicy.normalize(raw);
  if (normalized == null) return null;
  final uri = Uri.parse(normalized);
  return uri.scheme == 'wss' && _isPublicDomain(uri.host) ? normalized : null;
}

bool _isPublicDomain(String host) {
  final value = host.toLowerCase();
  if (!_isPlainDnsName(value)) return false;
  return !_privateSuffixes.any(value.endsWith) && !_looksLikeIpv4(value);
}

bool _isPlainDnsName(String host) =>
    host.contains('.') && !host.contains(':') && !host.endsWith('.');

bool _looksLikeIpv4(String host) {
  return host.codeUnits.every((unit) => unit == 46 || unit >= 48 && unit <= 57);
}
