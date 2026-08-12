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
  test('serializes remote and local mutations for one account', () async {
    final remote = _DelayedSocial();
    final local = _MemoryStore();
    final graph = SocialGraphCache(remote, local, RecordingFailureReporter());
    final firstProfile = ProfileId.parse('first');
    final secondProfile = ProfileId.parse('second');

    final first = graph.toggleFollow(firstProfile);
    await remote.firstStarted.future;
    final second = graph.toggleFollow(secondProfile);
    await Future<void>.delayed(Duration.zero);

    expect(remote.calls, 1);
    remote.release.complete();
    await Future.wait(<Future<bool>>[first, second]);
    expect(local.followed, {firstProfile, secondProfile});
  });
}

class _DelayedSocial implements NostrSocialPort {
  final firstStarted = Completer<void>();
  final release = Completer<void>();
  final Set<ProfileId> followed = <ProfileId>{};
  var calls = 0;

  @override
  NostrPublicKeyHex get accountPublicKey =>
      NostrPublicKeyHex.parse(testViewerPublicKey);

  @override
  NostrSocialPort snapshotForActiveAccount() => this;

  @override
  Future<FollowOutcome> follow(ProfileId profileId) async =>
      followed.add(profileId)
      ? FollowOutcome.newlyFollowed
      : FollowOutcome.alreadyFollowing;

  @override
  Future<bool> toggleFollow(ProfileId profileId) async {
    calls += 1;
    if (calls == 1) {
      firstStarted.complete();
      await release.future;
    }
    followed.add(profileId);
    return true;
  }

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() async => {...followed};

  @override
  Future<Set<ProfileId>> loadBlockedProfiles() async => <ProfileId>{};

  @override
  Future<bool> toggleBlock(ProfileId profileId) async => true;
}

class _MemoryStore implements SocialGraphStore {
  final Set<ProfileId> followed = <ProfileId>{};

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
    followed
      ..clear()
      ..addAll(profileIds);
  }

  @override
  Future<Set<ProfileId>> loadBlockedProfiles() async => <ProfileId>{};

  @override
  Future<void> saveBlockedProfiles(Set<ProfileId> profileIds) async {}
}
