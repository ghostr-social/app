import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_post_mapper.dart';
import 'package:ghostr/features/video_catalog/domain/video_repost_attribution.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

import '../support/nostr_test_values.dart';
import '../support/repost_samples.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  test('maps original creator separately from outer repost provenance', () {
    final source = repostablePost().nostrReference!.signedEvent!.value;
    final row = FfiFeedPost(
      postId: 'scope',
      eventId: testEventId,
      eventKind: 1,
      createdAt: BigInt.from(10),
      feedSortAt: BigInt.from(30),
      signedEventJson: source,
      isProtected: false,
      repost: FfiFeedRepost(
        eventId: secondTestEventId,
        eventKind: 6,
        target: FfiFeedRepostTarget.specificEvent,
        repostedAt: BigInt.from(30),
        reposter: rustFeedCreator(
          pubkey: testViewerPublicKey,
          displayName: 'Bob Relay',
          handle: '@bob',
        ),
      ),
      caption: 'clip',
      hashtags: const [],
      creator: rustFeedCreator(),
      media: rustFeedMedia(),
    );

    final post = const RustFeedPostMapper().map(row);

    expect(post.creator.displayName, 'Nora Relay');
    expect(post.repost?.reposter.displayName, 'Bob Relay');
    expect(post.repost?.target, VideoRepostTarget.specificEvent);
    expect(
      post.publishedAt,
      DateTime.fromMillisecondsSinceEpoch(10000, isUtc: true),
    );
    expect(
      post.feedActivityAt,
      DateTime.fromMillisecondsSinceEpoch(30000, isUtc: true),
    );
    expect(post.nostrReference?.signedEvent?.value, source);
  });
}
