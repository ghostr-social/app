import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/social/data/social_graph_cache.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/fake_nostr_social_port.dart';
import '../support/memory_social_graph_store.dart';
import '../support/recording_failure_reporter.dart';

void main() {
  test('an unexpected social read failure is evicted and can retry', () async {
    final remote = _FailingSocial();
    final graph = SocialGraphCache(
      remote,
      MemorySocialGraphStore(accountPublicKey: remote.accountPublicKey),
      RecordingFailureReporter(),
    );

    await expectLater(graph.loadBlockedProfiles(), throwsStateError);
    await expectLater(graph.loadBlockedProfiles(), throwsStateError);

    expect(remote.calls, 2);
  });
}

final class _FailingSocial extends FakeNostrSocialPort {
  var calls = 0;

  @override
  Future<Set<ProfileId>> loadBlockedProfiles() {
    calls += 1;
    throw StateError('unexpected remote failure');
  }
}
