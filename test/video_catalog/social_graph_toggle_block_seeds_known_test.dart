import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/social/data/social_graph_cache.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/fake_nostr_social_port.dart';
import '../support/memory_social_graph_store.dart';
import '../support/recording_failure_reporter.dart';

void main() {
  test('blocking hands the mirrored blocks to the relay write', () async {
    final remote = FakeNostrSocialPort();
    final mirrored = ProfileId.parse('mirrored-block');
    final target = ProfileId.parse('new-block');
    final local = MemorySocialGraphStore(
      accountPublicKey: remote.accountPublicKey,
      blocked: {mirrored},
    );
    final cache = SocialGraphCache(remote, local, RecordingFailureReporter());

    expect(await cache.toggleBlock(target), isTrue);

    expect(remote.lastKnownBlocked, {mirrored});
    expect(remote.blockedProfiles, {mirrored, target});
    expect(local.blocked, {mirrored, target});
  });
}
