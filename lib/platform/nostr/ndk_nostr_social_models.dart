part of 'ndk_nostr_social.dart';

class _NdkSocialTransport {
  const _NdkSocialTransport(this.events, this.broadcast);

  final NostrEventClient events;
  final SignedEventBroadcastPort broadcast;
}

class _NdkSocialScope {
  const _NdkSocialScope(this.state, this.signer, this.publicKey);

  final _NdkSocialState state;
  final EventSigner? signer;
  final String? publicKey;
}

class _NdkSocialState {
  final queue = KeyedSerialTaskQueue();
  final contactFloors = <String, ContactList>{};
  final muteFloors = <String, Nip51List>{};
  final lastTimestamps = <(String, int), int>{};
}

final class _SocialRecordKey {
  factory _SocialRecordKey.parse(int kind, String publicKey) {
    return _SocialRecordKey._(
      NostrEventKind.parse(kind),
      NostrPublicKeyHex.parse(publicKey),
    );
  }

  const _SocialRecordKey._(this.kind, this.publicKey);

  final NostrEventKind kind;
  final NostrPublicKeyHex publicKey;

  bool matches(NostrEventRecord record) {
    return record.kind == kind && record.authorPublicKeyHex == publicKey;
  }
}

NostrEventQuery _socialQuery(_SocialRecordKey key) {
  return NostrEventQuery(
    kinds: [key.kind.value],
    scope: NostrEventQueryScope.parse(authors: [key.publicKey.value]),
    limit: 1,
  );
}

NostrEventRecord? _newestSocialRecord(
  List<NostrEventRecord> records,
  _SocialRecordKey key,
) {
  return records.where(key.matches).fold<NostrEventRecord?>(
        null,
        _newerSocialRecord,
      );
}

NostrEventRecord _newerSocialRecord(
  NostrEventRecord? current,
  NostrEventRecord candidate,
) {
  if (current == null || candidate.createdAt > current.createdAt) {
    return candidate;
  }
  return current;
}

Nip01Event _localEvent(NostrEventRecord record) {
  return Nip01Event(
    id: record.id.value,
    pubKey: record.authorPublicKeyHex.value,
    createdAt: record.createdAt,
    kind: record.kind.value,
    tags: record.tags.toRaw(),
    content: record.content,
  );
}

ContactList _copyContactList(ContactList source) {
  return ContactList(pubKey: source.pubKey, contacts: List.of(source.contacts))
    ..contactRelays = List.of(source.contactRelays)
    ..petnames = List.of(source.petnames)
    ..followedTags = List.of(source.followedTags)
    ..followedCommunities = List.of(source.followedCommunities)
    ..followedEvents = List.of(source.followedEvents)
    ..createdAt = source.createdAt
    ..loadedTimestamp = source.loadedTimestamp
    ..sources = List.of(source.sources);
}

Nip51List _copyNip51List(Nip51List source) {
  return Nip51List(
    pubKey: source.pubKey,
    kind: source.kind,
    createdAt: source.createdAt,
    elements: source.elements.map(_copyNip51Element).toList(),
  );
}

Nip51ListElement _copyNip51Element(Nip51ListElement source) {
  return Nip51ListElement(
    tag: source.tag,
    value: source.value,
    private: source.private,
  );
}

ContactList? _newestContact(ContactList? first, ContactList? second) {
  if (first == null) return second;
  if (second == null) return first;
  return second.createdAt > first.createdAt ? second : first;
}

Nip51List? _newestMute(Nip51List? first, Nip51List? second) {
  if (first == null) return second;
  if (second == null) return first;
  return second.createdAt > first.createdAt ? second : first;
}

ContactList? _rememberContactFloor(
  _NdkSocialState state,
  String publicKey,
  ContactList? fetched,
) {
  final newest = _newestContact(state.contactFloors[publicKey], fetched);
  if (newest == null) return null;
  final floor = _copyContactList(newest);
  state.contactFloors[publicKey] = floor;
  return _copyContactList(floor);
}

Nip51List? _rememberMuteFloor(
  _NdkSocialState state,
  String publicKey,
  Nip51List? fetched,
) {
  final newest = _newestMute(state.muteFloors[publicKey], fetched);
  if (newest == null) return null;
  final floor = _copyNip51List(newest);
  state.muteFloors[publicKey] = floor;
  return _copyNip51List(floor);
}
