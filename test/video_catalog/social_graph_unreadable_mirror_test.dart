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
  test('an unreadable mirror still lets the relay list through', () async {
    final blocked = ProfileId.parse('npub1blocked');
    final reporter = RecordingFailureReporter();
    final cache = SocialGraphCache(
      FakeNostrSocialPort(blockedProfiles: {blocked}),
      const _UnreadableStore(),
      reporter,
    );

    expect(await cache.loadBlockedProfiles(), {blocked});
    expect(reporter.sources, ['SocialGraphCache.loadBlockedProfiles']);
  });
}

class _UnreadableStore implements SocialGraphStore {
  const _UnreadableStore();

  @override
  NostrPublicKeyHex get accountPublicKey {
    return NostrPublicKeyHex.parse(testViewerPublicKey);
  }

  @override
  SocialGraphStore snapshotForActiveAccount() => this;

  @override
  Future<Set<ProfileId>> loadBlockedProfiles() {
    throw const AppFailure('cache unreadable');
  }

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() {
    throw const AppFailure('cache unreadable');
  }

  @override
  Future<void> saveBlockedProfiles(Set<ProfileId> profileIds) async {}

  @override
  Future<void> saveFollowedProfiles(Set<ProfileId> profileIds) async {}
}
