import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/social/data/social_graph_cache.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/fake_nostr_social_port.dart';
import '../support/memory_social_graph_store.dart';
import '../support/recording_failure_reporter.dart';

void main() {
  test('relay reads widen the mirror instead of replacing it', () async {
    final mirrored = ProfileId.parse('mirrored-block');
    final relayed = ProfileId.parse('relayed-block');
    final remote = FakeNostrSocialPort(blockedProfiles: {relayed});
    final local = MemorySocialGraphStore(
      accountPublicKey: remote.accountPublicKey,
      blocked: {mirrored},
    );
    final cache = SocialGraphCache(remote, local, RecordingFailureReporter());

    expect(await cache.loadBlockedProfiles(), {mirrored, relayed});
    expect(local.blocked, {mirrored, relayed});
  });
}
