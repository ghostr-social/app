import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';
import 'package:ndk/ndk.dart';

import '../support/fake_rust_feed_port.dart';
import '../support/nostr_test_values.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  // Parity spec: nostr_video_event_mapper.dart keeps the Nostr event id
  // as the post id and creator_profile_summary.dart keys profiles by
  // npub — the Rust rows must land on the same domain identities.
  test('maps rust feed rows onto domain posts in feed order', () async {
    final port = FakeRustFeedPort(updates: [
      rustFeedBaseline(),
      rustFeedUpdate(revision: 1, posts: [
        rustFeedPost(
          eventId: testEventId,
          createdAt: 1754000000,
          caption: 'Ghost tape #9',
          hashtags: const ['ghostr'],
          creator: rustFeedCreator(
            pubkey: testCreatorPublicKey,
            displayName: 'Nora Relay',
            handle: '@norarelay',
            avatarUrl: 'https://cdn.example/nora.png',
          ),
        ),
        rustFeedPost(eventId: secondTestEventId, createdAt: 1753990000),
      ]),
    ]);
    final source = RustFeedRemoteSource(port: port);

    final posts = await source.loadRemoteFeed(searchQuery: 'ghost');

    expect(posts, hasLength(2));
    final post = posts.first;
    expect(post.id.value, testEventId);
    expect(posts.last.id.value, secondTestEventId);
    expect(post.caption, 'Ghost tape #9');
    expect(post.hashtags, const ['ghostr']);
    expect(post.songName, 'Original sound');
    expect(post.creator.id.value, Nip19.encodePubKey(testCreatorPublicKey));
    expect(post.creator.displayName, 'Nora Relay');
    expect(post.creator.handle, '@norarelay');
    expect(post.creator.avatarUrl, 'https://cdn.example/nora.png');
    expect(
      post.publishedAt,
      DateTime.fromMillisecondsSinceEpoch(1754000000 * 1000, isUtc: true),
    );
    expect(post.likeCount, 0);
    expect(post.commentCount, 0);
    expect(post.viewerHasLiked, isFalse);
    expect(port.closedFeedIds, [port.feedId]);
  });
}
