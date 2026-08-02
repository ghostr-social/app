import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/engagement/domain/video_engagement.dart';
import 'package:ghostr/features/comments/domain/video_comment.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';

import '../support/fakes.dart';
import '../support/hybrid_repository_harness.dart';
import '../support/nostr_reference.dart';
import '../support/sample_data.dart';

void main() {
  test('hydrates and toggles video likes through the Nostr port', () async {
    final reference = nostrReference(
      eventId: testEventId,
    );
    final engagement = FakeNostrEngagementPort()
      ..engagements[testEventId] = VideoEngagement(
        likeCount: 9,
        viewerHasLiked: false,
      );
    final comments = FakeNostrCommentsPort()
      ..comments.add(VideoComment(
        identity: VideoCommentIdentity.parse(
          id: secondTestEventId,
          authorPublicKeyHex: testAuthorPublicKey,
        ),
        text: VideoCommentText(
          authorLabel: 'Author',
          content: 'Comment',
        ),
        createdAt: DateTime(2026, 8, 2),
      ));
    final harness = await buildHybridRepositoryHarness(
      FakeRemoteVideoSource([
        samplePost(nostrReference: reference),
      ]),
      ports: HybridHarnessPorts(
        engagement: engagement,
        comments: comments,
      ),
    );

    final post = (await harness.feed.loadFeed(FeedKind.forYou)).single;
    final liked = await harness.engagement.toggleLike(post);

    expect(post.likeCount, 9);
    expect(post.viewerHasLiked, isFalse);
    expect(post.commentCount, 1);
    expect(liked.likeCount, 10);
    expect(liked.viewerHasLiked, isTrue);
  });
}
