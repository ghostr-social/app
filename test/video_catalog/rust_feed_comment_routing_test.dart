import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';

import '../support/fakes.dart';
import '../support/hybrid_repository_harness.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  // Commenting needs the parent reference; without it the catalog
  // refuses the write outright (nostr_video_interactions.dart).
  test('routes a comment on a rust-served row to the comments port', () async {
    final comments = FakeNostrCommentsPort();
    final harness = await buildHybridRepositoryHarness(
      rustFeedSourceServing([
        rustFeedPost(
          eventKind: 34235,
          details: const RustFeedPostDetails(identifier: 'clip-1'),
        ),
      ]),
      ports: HybridHarnessPorts(comments: comments),
    );

    final post = (await harness.feed.loadFeed(FeedKind.forYou)).single;
    final published = await harness.comments.publishComment(
      post: post,
      content: 'Relay-side banger indeed',
    );

    expect(comments.comments, [published]);
    expect(await harness.comments.loadComments(post), [published]);
  });
}
