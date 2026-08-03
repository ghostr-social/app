import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/social/data/social_graph_cache.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/fake_nostr_social_port.dart';
import '../support/memory_social_graph_store.dart';
import '../support/nostr_test_values.dart';
import '../support/recording_failure_reporter.dart';

void main() {
  test('projects an accepted block into stale local fallback', () async {
    final profile = ProfileId.parse('creator');
    final remote = FakeNostrSocialPort();
    final local = MemorySocialGraphStore(
      accountPublicKey: NostrPublicKeyHex.parse(testViewerPublicKey),
      rejectBlockWrites: true,
    );
    final graph = SocialGraphCache(remote, local, RecordingFailureReporter());

    expect(await graph.toggleBlock(profile), isTrue);
    remote.loadFailure = const AppFailure('relay offline');

    expect(await graph.loadBlockedProfiles(), {profile});
  });
}
