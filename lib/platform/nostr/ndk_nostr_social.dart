import 'package:ghostr/core/async/keyed_serial_task_queue.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/time/clock.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/features/social/domain/nostr_social_port.dart';
import 'package:ghostr/features/social/domain/signed_event_broadcast_port.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/platform/nostr/ndk_broadcast_adapter.dart';
import 'package:ghostr/platform/nostr/signed_nostr_event_json.dart';
import 'package:ndk/ndk.dart';

part 'ndk_nostr_social_models.dart';
part 'ndk_nostr_social_mutations.dart';

class NdkNostrSocial implements NostrSocialPort {
  NdkNostrSocial({
    required Ndk ndk,
    required List<RelayUrl> relays,
    SignedEventBroadcastPort? broadcast,
    Clock clock = systemClock,
  })  : _broadcastPort =
            broadcast ?? NdkBroadcastAdapter(ndk: ndk, relays: relays),
        _ndk = ndk,
        _clock = clock,
        _scope = _NdkSocialScope(_NdkSocialState(), null, null);

  NdkNostrSocial._(
    this._ndk,
    this._broadcastPort,
    this._clock,
    this._scope,
  );

  final Ndk _ndk;
  final SignedEventBroadcastPort _broadcastPort;
  final Clock _clock;
  final _NdkSocialScope _scope;
  _NdkSocialState get _state => _scope.state;
  EventSigner? get _signer => _scope.signer;
  String? get _publicKey => _scope.publicKey;

  @override
  NostrSocialPort snapshotForActiveAccount() {
    if (_signer != null) return this;
    final signer = _requireSigner();
    return NdkNostrSocial._(
      _ndk,
      _broadcastPort,
      _clock,
      _NdkSocialScope(_state, signer, signer.getPublicKey()),
    );
  }

  @override
  NostrPublicKeyHex get accountPublicKey {
    return NostrPublicKeyHex.parse(_publicKey ?? _requirePublicKey());
  }

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
    return _guard('Could not update the Nostr mute list.', () {
      final target = _decodeProfile(profileId);
      return _activeScope._enqueueBlock(target);
    });
  }

  @override
  Future<bool> toggleFollow(ProfileId profileId) {
    return _guard('Could not update the Nostr follow list.', () {
      final target = _decodeProfile(profileId);
      return _activeScope._enqueueFollow(target);
    });
  }

  Future<Set<ProfileId>> _loadBlockedProfiles() async {
    final publicKey = _publicKey ?? _requirePublicKey();
    _requireActiveAccount(publicKey);
    final fetched = await _ndk.lists.getSingleNip51List(Nip51List.kMute);
    _requireActiveAccount(publicKey);
    final list = _rememberMuteFloor(_state, publicKey, fetched);
    return list?.pubKeys.map(_encodedProfile).toSet() ?? <ProfileId>{};
  }

  Future<Set<ProfileId>> _loadFollowedProfiles() async {
    final publicKey = _publicKey ?? _requirePublicKey();
    final fetched = await _ndk.follows.getContactList(publicKey);
    final contacts = _rememberContactFloor(_state, publicKey, fetched);
    return contacts?.contacts
            .map(Nip19.encodePubKey)
            .map(ProfileId.parse)
            .toSet() ??
        <ProfileId>{};
  }

  NdkNostrSocial get _activeScope {
    return snapshotForActiveAccount() as NdkNostrSocial;
  }

  bool _isActiveAccount(String publicKey) {
    return _ndk.accounts.getPublicKey() == publicKey;
  }

  void _requireActiveAccount(String publicKey) {
    if (!_isActiveAccount(publicKey)) {
      throw const AppFailure('The active account changed. Try again.');
    }
  }

  EventSigner _requireSigner() {
    final signer = _ndk.accounts.getLoggedAccount()?.signer;
    if (signer == null || !signer.canSign()) {
      throw const AppFailure('Sign in first.');
    }
    return signer;
  }

  String _requirePublicKey() {
    final publicKey = _ndk.accounts.getPublicKey();
    if (publicKey == null) throw const AppFailure('Sign in first.');
    return publicKey;
  }

  ProfileId _encodedProfile(Nip51ListElement element) {
    return ProfileId.parse(Nip19.encodePubKey(element.value));
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
