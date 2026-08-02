import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';

import '../support/fakes.dart';
import '../support/hybrid_repository_harness.dart';
import '../support/sample_data.dart';

void main() {
  test('removes blocked Nostr accounts from every video feed', () async {
    final blocked = sampleCreator(id: 'npub1blocked');
    final visible = sampleCreator(id: 'npub1visible');
    final harness = await buildHybridRepositoryHarness(
      FakeRemoteVideoSource([
        samplePost(creator: blocked),
        samplePost(id: 'visible', creator: visible),
      ]),
      ports: HybridHarnessPorts(
        social: FakeNostrSocialPort(blockedProfiles: {blocked.id}),
      ),
    );

    final posts = await harness.feed.loadFeed(FeedKind.forYou);

    expect(posts.map((post) => post.creator.id), [visible.id]);
  });
}
