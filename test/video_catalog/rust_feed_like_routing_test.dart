import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/engagement/domain/nostr_engagement_port.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';

import '../support/fakes.dart';
import '../support/hybrid_repository_harness.dart';
import '../support/nostr_test_values.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  // A like on a Rust-served row must reach the same relay write the
  // ndk pipeline makes: the port is keyed by the row's reference.
  test('routes a like on a rust-served row to the engagement port', () async {
    final engagement = FakeNostrEngagementPort();
    final harness = await buildHybridRepositoryHarness(
      rustFeedSourceServing([
        rustFeedPost(
          eventKind: 34235,
          details: const RustFeedPostDetails(identifier: 'clip-1'),
        ),
      ]),
      ports: HybridHarnessPorts(engagement: engagement),
    );

    final post = (await harness.feed.loadFeed(FeedKind.forYou)).single;
    final liked = await harness.engagement.toggleLike(post);

    expect(engagement.intents, [VideoLikeIntent.like]);
    expect(engagement.engagements[testEventId]?.viewerHasLiked, isTrue);
    expect(liked.likeCount, 1);
    expect(liked.viewerHasLiked, isTrue);
  });
}
