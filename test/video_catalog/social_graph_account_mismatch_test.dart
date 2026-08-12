import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/social/data/social_graph_cache.dart';
import 'package:ghostr/features/social/domain/follow_outcome.dart';
import 'package:ghostr/features/social/domain/nostr_social_port.dart';
import 'package:ghostr/features/social/domain/social_graph_store.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/nostr_test_values.dart';
import '../support/recording_failure_reporter.dart';

void main() {
  test('rejects mismatched remote and local account snapshots', () async {
    final remote = _ScopedSocial(testAuthorPublicKey);
    final local = _ScopedStore(testViewerPublicKey);
    final graph = SocialGraphCache(remote, local, RecordingFailureReporter());

    await expectLater(
      graph.loadFollowedProfiles(),
      throwsA(
        isA<AppFailure>().having(
          (failure) => failure.message,
          'message',
          'The active account changed. Try again.',
        ),
      ),
    );

    expect(remote.loads, 0);
  });
}

class _ScopedSocial implements NostrSocialPort {
  _ScopedSocial(String publicKey)
    : accountPublicKey = NostrPublicKeyHex.parse(publicKey);

  @override
  final NostrPublicKeyHex accountPublicKey;
  var loads = 0;

  @override
  NostrSocialPort snapshotForActiveAccount() => this;

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() async {
    loads += 1;
    return <ProfileId>{};
  }

  @override
  Future<Set<ProfileId>> loadBlockedProfiles() async => <ProfileId>{};

  @override
  Future<FollowOutcome> follow(ProfileId profileId) async =>
      FollowOutcome.newlyFollowed;

  @override
  Future<bool> toggleBlock(ProfileId profileId) async => true;

  @override
  Future<bool> toggleFollow(ProfileId profileId) async => true;
}

class _ScopedStore implements SocialGraphStore {
  _ScopedStore(String publicKey)
    : accountPublicKey = NostrPublicKeyHex.parse(publicKey);

  @override
  final NostrPublicKeyHex accountPublicKey;

  @override
  SocialGraphStore snapshotForActiveAccount() => this;

  @override
  Future<Set<ProfileId>> loadBlockedProfiles() async => <ProfileId>{};

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() async => <ProfileId>{};

  @override
  Future<void> saveBlockedProfiles(Set<ProfileId> profileIds) async {}

  @override
  Future<void> saveFollowedProfiles(Set<ProfileId> profileIds) async {}
}
