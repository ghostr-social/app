import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/social/data/social_graph_cache.dart';
import 'package:ghostr/features/social/domain/social_graph_store.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/fake_nostr_social_port.dart';
import '../support/nostr_test_values.dart';
import '../support/recording_failure_reporter.dart';

void main() {
  test('returns relay social lists when cache writes fail', () async {
    final followed = ProfileId.parse('npub1followed');
    final blocked = ProfileId.parse('npub1blocked');
    final stale = ProfileId.parse('npub1stale');
    final reporter = RecordingFailureReporter();
    final cache = SocialGraphCache(
      FakeNostrSocialPort(
        followedProfiles: {followed},
        blockedProfiles: {blocked},
      ),
      _StaleRejectingStore(stale),
      reporter,
    );

    expect(await cache.loadFollowedProfiles(), {followed});
    expect(await cache.loadBlockedProfiles(), {blocked});
    expect(reporter.sources, [
      'SocialGraphCache.cacheFollow',
      'SocialGraphCache.cacheBlock',
    ]);
  });
}

class _StaleRejectingStore implements SocialGraphStore {
  const _StaleRejectingStore(this.stale);

  final ProfileId stale;

  @override
  NostrPublicKeyHex get accountPublicKey {
    return NostrPublicKeyHex.parse(testViewerPublicKey);
  }

  @override
  SocialGraphStore snapshotForActiveAccount() => this;

  @override
  Future<Set<ProfileId>> loadBlockedProfiles() async => {stale};

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() async => {stale};

  @override
  Future<void> saveBlockedProfiles(Set<ProfileId> profileIds) {
    throw const AppFailure('cache write failed');
  }

  @override
  Future<void> saveFollowedProfiles(Set<ProfileId> profileIds) {
    throw const AppFailure('cache write failed');
  }
}
