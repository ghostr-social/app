part of 'ndk_nostr_social.dart';

extension _NdkNostrSocialMutations on NdkNostrSocial {
  Future<bool> _enqueueFollow(String target) {
    final publicKey = _publicKey!;
    return _state.queue.run(
      (publicKey, ContactList.kKind),
      () => _toggleFollowTarget(publicKey, target),
    );
  }

  Future<bool> _enqueueBlock(String target) {
    final publicKey = _publicKey!;
    return _state.queue.run(
      (publicKey, Nip51List.kMute),
      () => _toggleBlockTarget(publicKey, target),
    );
  }

  Future<bool> _toggleFollowTarget(String publicKey, String target) async {
    final contacts = await _followBaseline(publicKey);
    final isFollowing = contacts.contacts.contains(target);
    if (isFollowing) {
      _removeAllContacts(contacts, target);
    } else {
      _addContact(contacts, target);
    }
    contacts
      ..pubKey = publicKey
      ..createdAt = _nextTimestamp(publicKey, ContactList.kKind, contacts)
      ..loadedTimestamp = _clock().millisecondsSinceEpoch ~/ 1000;
    final accepted = await _broadcast(contacts.toEvent());
    _state.contactFloors[publicKey] = _copyContactList(contacts);
    await _cacheAcceptedFollow(contacts, accepted);
    return !isFollowing;
  }

  Future<bool> _toggleBlockTarget(String publicKey, String target) async {
    final list = await _muteBaseline(publicKey);
    final isBlocked = list.pubKeys.any((item) => item.value == target);
    if (isBlocked) {
      list.removeElement(Nip51List.kPubkey, target);
    } else {
      list.addElement(Nip51List.kPubkey, target, true);
    }
    list
      ..pubKey = publicKey
      ..createdAt = _nextTimestamp(publicKey, Nip51List.kMute, list);
    final accepted = await _broadcast(await list.toEvent(_signer));
    _state.muteFloors[publicKey] = _copyNip51List(list);
    await _cacheEvent(accepted);
    return !isBlocked;
  }

  Future<ContactList> _followBaseline(String publicKey) async {
    final accepted = _state.contactFloors[publicKey];
    if (!_isActiveAccount(publicKey)) {
      if (accepted == null) {
        throw const AppFailure('The active account changed.');
      }
      return _copyContactList(accepted);
    }
    final key = _SocialRecordKey.parse(ContactList.kKind, publicKey);
    final records = await _transport.events.query(_socialQuery(key));
    final record = _newestSocialRecord(records, key);
    final remote =
        record == null ? null : ContactList.fromEvent(_localEvent(record));
    final newest = _newestContact(accepted, remote);
    return newest == null
        ? ContactList(pubKey: publicKey, contacts: <String>[])
        : _copyContactList(newest);
  }

  Future<Nip51List> _muteBaseline(String publicKey) async {
    final accepted = _state.muteFloors[publicKey];
    if (!_isActiveAccount(publicKey)) return _acceptedMute(publicKey, accepted);
    final key = _SocialRecordKey.parse(Nip51List.kMute, publicKey);
    final records = await _transport.events.query(_socialQuery(key));
    if (!_isActiveAccount(publicKey)) return _acceptedMute(publicKey, accepted);
    final record = _newestSocialRecord(records, key);
    final remote = record == null
        ? null
        : await Nip51List.fromEvent(_localEvent(record), _signer);
    if (!_isActiveAccount(publicKey)) return _acceptedMute(publicKey, accepted);
    final newest = _newestMute(accepted, remote);
    return newest == null ? _emptyMute(publicKey) : _copyNip51List(newest);
  }

  Nip51List _acceptedMute(String publicKey, Nip51List? accepted) {
    if (accepted == null) throw const AppFailure('The active account changed.');
    return _copyNip51List(accepted);
  }

  Nip51List _emptyMute(String publicKey) {
    return Nip51List(
      pubKey: publicKey,
      kind: Nip51List.kMute,
      createdAt: 0,
      elements: <Nip51ListElement>[],
    );
  }

  int _nextTimestamp(String publicKey, int kind, Object source) {
    final baseline = source is ContactList
        ? source.createdAt
        : (source as Nip51List).createdAt;
    final key = (publicKey, kind);
    final last = _state.lastTimestamps[key] ?? 0;
    final now = _clock().millisecondsSinceEpoch ~/ 1000;
    final next =
        <int>[now, baseline + 1, last + 1].reduce((a, b) => a > b ? a : b);
    _state.lastTimestamps[key] = next;
    return next;
  }

  void _removeAllContacts(ContactList contacts, String target) {
    for (var index = contacts.contacts.length - 1; index >= 0; index -= 1) {
      if (contacts.contacts[index] != target) continue;
      contacts.contacts.removeAt(index);
      if (index < contacts.contactRelays.length) {
        contacts.contactRelays.removeAt(index);
      }
      if (index < contacts.petnames.length) contacts.petnames.removeAt(index);
    }
  }

  void _addContact(ContactList contacts, String target) {
    contacts.contacts.add(target);
    contacts.contactRelays.add('');
    contacts.petnames.add('');
  }

  /// Signs in Dart, then hands the canonical NIP-01 JSON to the
  /// transport. The accepted event returns for the local caches.
  Future<Nip01Event> _broadcast(Nip01Event event) async {
    final signed = await _signer!.sign(event);
    await _transport.broadcast.broadcast(encodeSignedNostrEvent(signed));
    return signed;
  }

  Future<void> _cacheContact(ContactList contacts) async {
    try {
      await _ndk.config.cache.saveContactList(_copyContactList(contacts));
    } on Object catch (error, stackTrace) {
      logBoundaryFailure(
        source: 'ghostr.nostr.social.cache-contact',
        message: 'An accepted contact list could not be cached locally.',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }

  Future<void> _cacheAcceptedFollow(
    ContactList contacts,
    Nip01Event event,
  ) async {
    await _cacheEvent(event);
    await _cacheContact(contacts);
  }

  Future<void> _cacheEvent(Nip01Event event) async {
    try {
      await _ndk.config.cache.saveEvent(event);
    } on Object catch (error, stackTrace) {
      logBoundaryFailure(
        source: 'ghostr.nostr.social.cache-event',
        message: 'An accepted social event could not be cached locally.',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }
}
