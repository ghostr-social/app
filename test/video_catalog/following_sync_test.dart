import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';

import '../support/fakes.dart';
import '../support/hybrid_repository_harness.dart';
import '../support/sample_data.dart';

void main() {
  test('builds the Following feed from the Nostr contact list', () async {
    final creator = sampleCreator(id: 'npub1followedcreator');
    final social = FakeNostrSocialPort(followedProfiles: {creator.id});
    final remote = FakeRemoteVideoSource([
      samplePost(creator: creator),
      samplePost(id: 'other', creator: sampleCreator(id: 'npub1other')),
    ]);
    final harness = await buildHybridRepositoryHarness(
      remote,
      ports: HybridHarnessPorts(
        social: social,
      ),
    );

    final posts = await harness.feed.loadFeed(FeedKind.following);

    expect(posts.map((post) => post.creator.id), [creator.id]);
    expect(remote.requestedCreatorIds, {creator.id});
  });
}
