import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';

import '../support/fake_rust_feed_port.dart';
import '../support/nostr_test_values.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  // ndk parity: NdkVideoRemoteSource skips malformed rows instead of
  // sinking the whole page (ndk_video_remote_source.dart).
  test('skips rows without playable media and keeps the rest', () async {
    final port = FakeRustFeedPort(updates: [
      rustFeedUpdate(revision: 0),
      rustFeedUpdate(revision: 1, posts: [
        rustFeedPost(
          eventId: testEventId,
          media: rustFeedMedia(urls: const []),
        ),
        rustFeedPost(eventId: secondTestEventId),
      ]),
    ]);
    final source = RustFeedRemoteSource(port: port);

    final posts = await source.loadRemoteFeed(searchQuery: 'ghost');

    expect(posts.single.id.value, secondTestEventId);
  });

  test('skips rows with an unknown delivery kind', () async {
    final port = FakeRustFeedPort(updates: [
      rustFeedUpdate(revision: 0),
      rustFeedUpdate(revision: 1, posts: [
        rustFeedPost(
          eventId: testEventId,
          media: rustFeedMedia(delivery: 'carrier-pigeon'),
        ),
        rustFeedPost(eventId: secondTestEventId),
      ]),
    ]);
    final source = RustFeedRemoteSource(port: port);

    final posts = await source.loadRemoteFeed(searchQuery: 'ghost');

    expect(posts.single.id.value, secondTestEventId);
  });
}
