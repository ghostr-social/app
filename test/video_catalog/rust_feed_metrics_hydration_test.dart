import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/comments/domain/video_comment.dart';
import 'package:ghostr/features/engagement/domain/video_engagement.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';

import '../support/fakes.dart';
import '../support/hybrid_repository_harness.dart';
import '../support/nostr_test_values.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  // Rust never aggregates reactions: the counts a row shows come from
  // the same relay enrichment pass the ndk rows go through
  // (HybridVideoReader._hydrate), which keys off the reference.
  test('hydrates rust-served rows with relay likes and comment counts',
      () async {
    final engagement = FakeNostrEngagementPort()
      ..engagements[testEventId] =
          VideoEngagement(likeCount: 9, viewerHasLiked: true);
    final comments = FakeNostrCommentsPort()..comments.add(_comment());
    final harness = await buildHybridRepositoryHarness(
      rustFeedSourceServing([rustFeedPost(eventKind: 22)]),
      ports: HybridHarnessPorts(engagement: engagement, comments: comments),
    );

    final post = (await harness.feed.loadFeed(FeedKind.forYou)).single;

    expect(post.likeCount, 9);
    expect(post.viewerHasLiked, isTrue);
    expect(post.commentCount, 1);
  });
}

VideoComment _comment() {
  return VideoComment(
    identity: VideoCommentIdentity.parse(
      id: secondTestEventId,
      authorPublicKeyHex: testAuthorPublicKey,
    ),
    text: VideoCommentText(authorLabel: 'Author', content: 'Comment'),
    createdAt: DateTime.utc(2026, 8, 2),
  );
}
