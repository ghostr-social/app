import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/features/social/domain/nostr_social_port.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ndk/ndk.dart';

class NdkNostrSocial implements NostrSocialPort {
  NdkNostrSocial({required Ndk ndk, required List<RelayUrl> relays})
      : _relayUrls = relays.map((relay) => relay.value).toList(),
        _ndk = ndk;

  final Ndk _ndk;
  final List<String> _relayUrls;

  @override
  Future<Set<ProfileId>> loadBlockedProfiles() {
    return _guard(
      'Could not synchronize the Nostr mute list.',
      _loadBlockedProfiles,
    );
  }

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() {
    return _guard(
      'Could not synchronize the Nostr follow list.',
      _loadFollowedProfiles,
    );
  }

  @override
  Future<bool> toggleBlock(ProfileId profileId) {
    return _guard(
      'Could not update the Nostr mute list.',
      () => _toggleBlock(profileId),
    );
  }

  @override
  Future<bool> toggleFollow(ProfileId profileId) {
    return _guard(
      'Could not update the Nostr follow list.',
      () => _toggleFollow(profileId),
    );
  }

  Future<Set<ProfileId>> _loadBlockedProfiles() async {
    final list = await _ndk.lists.getSingleNip51List(Nip51List.kMute);
    return list?.pubKeys.map(_encodedProfile).toSet() ?? <ProfileId>{};
  }

  Future<Set<ProfileId>> _loadFollowedProfiles() async {
    final contacts = await _ndk.follows.getContactList(_requirePublicKey());
    return contacts?.contacts
            .map(Nip19.encodePubKey)
            .map(ProfileId.parse)
            .toSet() ??
        <ProfileId>{};
  }

  ProfileId _encodedProfile(Nip51ListElement element) {
    return ProfileId.parse(Nip19.encodePubKey(element.value));
  }

  Future<bool> _toggleBlock(ProfileId profileId) async {
    final target = _decodeProfile(profileId);
    final list = await _ndk.lists.getSingleNip51List(Nip51List.kMute);
    final isBlocked =
        list?.pubKeys.any((item) => item.value == target) ?? false;
    return isBlocked ? _removeBlock(target) : _addBlock(target);
  }

  Future<bool> _removeBlock(String target) async {
    await _ndk.lists.removeElementFromList(
      kind: Nip51List.kMute,
      tag: Nip51List.kPubkey,
      value: target,
      broadcastRelays: _relayUrls,
    );
    return false;
  }

  Future<bool> _addBlock(String target) async {
    await _ndk.lists.addElementToList(
      kind: Nip51List.kMute,
      tag: Nip51List.kPubkey,
      value: target,
      broadcastRelays: _relayUrls,
      private: true,
    );
    return true;
  }

  Future<bool> _toggleFollow(ProfileId profileId) async {
    final target = _decodeProfile(profileId);
    final contacts = await _ndk.follows.getContactList(_requirePublicKey());
    if (contacts?.contacts.contains(target) ?? false) {
      await _ndk.follows.broadcastRemoveContact(
        target,
        customRelays: _relayUrls,
      );
      return false;
    }
    await _ndk.follows.broadcastAddContact(target, customRelays: _relayUrls);
    return true;
  }

  String _requirePublicKey() {
    final publicKey = _ndk.accounts.getPublicKey();
    if (publicKey == null) throw const AppFailure('Sign in first.');
    return publicKey;
  }

  String _decodeProfile(ProfileId profileId) {
    if (!profileId.startsWith('npub1')) {
      throw const AppFailure('This creator has no Nostr public key.');
    }
    return Nip19.decode(profileId.value);
  }

  Future<T> _guard<T>(String message, Future<T> Function() operation) async {
    try {
      return await operation();
    } on AppFailure {
      rethrow;
    } on Object catch (error, stackTrace) {
      throw translatedBoundaryFailure(
        source: 'ghostr.nostr.social',
        message: message,
        error: error,
        stackTrace: stackTrace,
      );
    }
  }
}
