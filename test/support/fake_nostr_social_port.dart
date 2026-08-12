import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/social/domain/follow_outcome.dart';
import 'package:ghostr/features/social/domain/nostr_social_port.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import 'nostr_test_values.dart';

typedef FakeActiveAccountReader = NostrPublicKeyHex Function();

class FakeNostrSocialPort implements NostrSocialPort {
  FakeNostrSocialPort({
    Set<ProfileId>? followedProfiles,
    Set<ProfileId>? blockedProfiles,
    FakeActiveAccountReader? activeAccount,
  }) : followedProfiles = followedProfiles ?? <ProfileId>{},
       blockedProfiles = blockedProfiles ?? <ProfileId>{},
       _activeAccount = activeAccount ?? _testAccount;

  final Set<ProfileId> followedProfiles;
  final Set<ProfileId> blockedProfiles;
  final FakeActiveAccountReader _activeAccount;
  AppFailure? loadFailure;
  AppFailure? toggleFailure;
  Set<ProfileId>? lastKnownBlocked;

  @override
  NostrPublicKeyHex get accountPublicKey => _activeAccount();

  @override
  NostrSocialPort snapshotForActiveAccount() =>
      _ScopedFakeNostrSocialPort(this, accountPublicKey);

  @override
  Future<Set<ProfileId>> loadBlockedProfiles() async {
    if (loadFailure case final failure?) throw failure;
    return {...blockedProfiles};
  }

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() async {
    if (loadFailure case final failure?) throw failure;
    return {...followedProfiles};
  }

  @override
  Future<FollowOutcome> follow(ProfileId profileId) async {
    if (toggleFailure case final failure?) throw failure;
    return followedProfiles.add(profileId)
        ? FollowOutcome.newlyFollowed
        : FollowOutcome.alreadyFollowing;
  }

  @override
  Future<bool> toggleFollow(ProfileId profileId) async {
    if (toggleFailure case final failure?) throw failure;
    if (followedProfiles.remove(profileId)) return false;
    followedProfiles.add(profileId);
    return true;
  }

  @override
  Future<bool> toggleBlock(
    ProfileId profileId, {
    Set<ProfileId> knownBlocked = const {},
  }) async {
    if (toggleFailure case final failure?) throw failure;
    blockedProfiles.addAll(knownBlocked);
    lastKnownBlocked = {...knownBlocked};
    if (blockedProfiles.remove(profileId)) return false;
    blockedProfiles.add(profileId);
    return true;
  }
}

class _ScopedFakeNostrSocialPort implements NostrSocialPort {
  const _ScopedFakeNostrSocialPort(this._delegate, this.accountPublicKey);

  final FakeNostrSocialPort _delegate;
  @override
  final NostrPublicKeyHex accountPublicKey;
  @override
  NostrSocialPort snapshotForActiveAccount() => this;
  @override
  Future<Set<ProfileId>> loadBlockedProfiles() =>
      _delegate.loadBlockedProfiles();

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() =>
      _delegate.loadFollowedProfiles();

  @override
  Future<FollowOutcome> follow(ProfileId profileId) =>
      _delegate.follow(profileId);

  @override
  Future<bool> toggleBlock(
    ProfileId profileId, {
    Set<ProfileId> knownBlocked = const {},
  }) =>
      _delegate.toggleBlock(profileId, knownBlocked: knownBlocked);

  @override
  Future<bool> toggleFollow(ProfileId profileId) =>
      _delegate.toggleFollow(profileId);
}

NostrPublicKeyHex _testAccount() =>
    NostrPublicKeyHex.parse(testViewerPublicKey);
