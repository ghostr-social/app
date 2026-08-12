import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/social/data/social_graph_cache.dart';
import 'package:ghostr/features/social/domain/follow_outcome.dart';
import 'package:ghostr/features/social/domain/nostr_social_port.dart';
import 'package:ghostr/features/social/domain/social_graph_store.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/recording_failure_reporter.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('a stale follow load cannot overwrite a later mutation', () async {
    final remote = _StaleLoadSocial();
    final local = _MemoryStore();
    final graph = SocialGraphCache(remote, local, RecordingFailureReporter());
    final profile = ProfileId.parse('creator');

    final load = graph.loadFollowedProfiles();
    await remote.started.future;
    final toggle = graph.toggleFollow(profile);
    await Future<void>.delayed(Duration.zero);
    remote.release.complete();
    await Future.wait(<Future<Object>>[load, toggle]);

    expect(await local.loadFollowedProfiles(), {profile});
  });
}

class _StaleLoadSocial implements NostrSocialPort {
  final started = Completer<void>();
  final release = Completer<void>();
  final followed = <ProfileId>{};

  @override
  NostrPublicKeyHex get accountPublicKey {
    return NostrPublicKeyHex.parse(testViewerPublicKey);
  }

  @override
  NostrSocialPort snapshotForActiveAccount() => this;

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() async {
    started.complete();
    await release.future;
    return <ProfileId>{};
  }

  @override
  Future<FollowOutcome> follow(ProfileId profileId) async {
    return followed.add(profileId)
        ? FollowOutcome.newlyFollowed
        : FollowOutcome.alreadyFollowing;
  }

  @override
  Future<bool> toggleFollow(ProfileId profileId) async {
    if (followed.remove(profileId)) return false;
    followed.add(profileId);
    return true;
  }

  @override
  Future<Set<ProfileId>> loadBlockedProfiles() async => <ProfileId>{};

  @override
  Future<bool> toggleBlock(
    ProfileId profileId, {
    Set<ProfileId> knownBlocked = const {},
  }) async =>
      true;
}

class _MemoryStore implements SocialGraphStore {
  Set<ProfileId> followed = <ProfileId>{};

  @override
  NostrPublicKeyHex get accountPublicKey {
    return NostrPublicKeyHex.parse(testViewerPublicKey);
  }

  @override
  SocialGraphStore snapshotForActiveAccount() => this;

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() async => {...followed};

  @override
  Future<void> saveFollowedProfiles(Set<ProfileId> profileIds) async {
    followed = {...profileIds};
  }

  @override
  Future<Set<ProfileId>> loadBlockedProfiles() async => <ProfileId>{};

  @override
  Future<void> saveBlockedProfiles(Set<ProfileId> profileIds) async {}
}
