import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/social/data/social_graph_cache.dart';
import 'package:ghostr/features/social/domain/social_graph_store.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/fake_nostr_social_port.dart';
import '../support/nostr_test_values.dart';
import '../support/recording_failure_reporter.dart';

void main() {
  test('keeps completed relay mutations when cache updates fail', () async {
    final remote = FakeNostrSocialPort();
    final reporter = RecordingFailureReporter();
    final cache = SocialGraphCache(
      remote,
      _FailingSocialGraphStore(),
      reporter,
    );
    final creator = ProfileId.parse('npub1creator');

    expect(await cache.toggleFollow(creator), isTrue);
    expect(await cache.toggleBlock(creator), isTrue);
    expect(remote.followedProfiles, {creator});
    expect(remote.blockedProfiles, {creator});
    expect(reporter.sources, [
      'SocialGraphCache.cacheFollow',
      'SocialGraphCache.cacheBlock',
    ]);
  });
}

class _FailingSocialGraphStore implements SocialGraphStore {
  @override
  NostrPublicKeyHex get accountPublicKey {
    return NostrPublicKeyHex.parse(testViewerPublicKey);
  }

  @override
  SocialGraphStore snapshotForActiveAccount() => this;

  @override
  Future<Set<ProfileId>> loadBlockedProfiles() async => <ProfileId>{};

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() {
    throw StateError('preferences unavailable');
  }

  @override
  Future<void> saveBlockedProfiles(Set<ProfileId> profileIds) async {
    throw StateError('preferences unavailable');
  }

  @override
  Future<void> saveFollowedProfiles(Set<ProfileId> profileIds) async {}
}
