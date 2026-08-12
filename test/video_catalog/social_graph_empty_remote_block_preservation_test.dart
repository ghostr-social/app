import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/social/data/social_graph_cache.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/fake_nostr_social_port.dart';
import '../support/memory_social_graph_store.dart';
import '../support/recording_failure_reporter.dart';

void main() {
  test('a quiet relay read never erases the mirrored block list', () async {
    final remote = FakeNostrSocialPort();
    final blocked = ProfileId.parse('blocked-creator');
    final local = MemorySocialGraphStore(
      accountPublicKey: remote.accountPublicKey,
      blocked: {blocked},
    );
    final cache = SocialGraphCache(remote, local, RecordingFailureReporter());

    expect(await cache.loadBlockedProfiles(), {blocked});
    expect(local.blocked, {blocked});
  });
}
