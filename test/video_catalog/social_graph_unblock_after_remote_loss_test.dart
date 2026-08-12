import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/social/data/social_graph_cache.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/fake_nostr_social_port.dart';
import '../support/memory_social_graph_store.dart';
import '../support/recording_failure_reporter.dart';

void main() {
  test('unblocking works even when the relay lost the mute list', () async {
    final remote = FakeNostrSocialPort();
    final blocked = ProfileId.parse('blocked-creator');
    final local = MemorySocialGraphStore(
      accountPublicKey: remote.accountPublicKey,
      blocked: {blocked},
    );
    final cache = SocialGraphCache(remote, local, RecordingFailureReporter());

    expect(await cache.toggleBlock(blocked), isFalse);

    expect(remote.blockedProfiles, isEmpty);
    expect(local.blocked, isEmpty);
    expect(await cache.loadBlockedProfiles(), isEmpty);
  });
}
