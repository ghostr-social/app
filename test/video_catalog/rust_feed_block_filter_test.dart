import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ndk/ndk.dart';

import '../support/fakes.dart';
import '../support/hybrid_repository_harness.dart';
import '../support/nostr_test_values.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  // Blocks are creator-scoped (VideoFeedPolicy filters on
  // post.creator.id), so a Rust row must carry the npub identity the
  // block list is written in.
  test('keeps blocked creators out of a rust-served feed', () async {
    final blocked = ProfileId.parse(Nip19.encodePubKey(testCreatorPublicKey));
    final harness = await buildHybridRepositoryHarness(
      rustFeedSourceServing([rustFeedPost(eventKind: 22)]),
      ports: HybridHarnessPorts(
        social: FakeNostrSocialPort(blockedProfiles: {blocked}),
      ),
    );

    expect(await harness.feed.loadFeed(FeedKind.forYou), isEmpty);
  });
}
