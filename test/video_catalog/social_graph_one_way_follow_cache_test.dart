import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/social/data/social_graph_cache.dart';
import 'package:ghostr/features/social/domain/follow_outcome.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/fake_nostr_social_port.dart';
import '../support/memory_social_graph_store.dart';
import '../support/recording_failure_reporter.dart';

void main() {
  test('projects an accepted one-way follow over a stale relay read', () async {
    final remote = FakeNostrSocialPort();
    final local = MemorySocialGraphStore(
      accountPublicKey: remote.accountPublicKey,
    );
    final graph = SocialGraphCache(remote, local, RecordingFailureReporter());
    final creator = ProfileId.parse('creator');

    expect(await graph.follow(creator), FollowOutcome.newlyFollowed);
    expect(await local.loadFollowedProfiles(), {creator});
    remote.followedProfiles.clear();

    expect(await graph.loadFollowedProfiles(), {creator});
  });
}
