import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/nostr_test_values.dart';
import '../support/rust_feed_fixtures.dart';
import '../support/timed_rust_feed_port.dart';

void main() {
  final cursor = DateTime.fromMillisecondsSinceEpoch(
    1754000000 * 1000,
    isUtc: true,
  );

  // The end of a feed is an answer, not a hang: an older page that adds
  // no rows still settles its revision (rust/src/api/feed_state.rs), so
  // the request ends there instead of burning the whole deadline.
  test('ends the older page as soon as a spent revision settles', () {
    fakeAsync((async) {
      final loaded = rustFeedPost(eventId: testEventId, createdAt: 1754005000);
      final port = TimedRustFeedPort([
        (
          at: const Duration(seconds: 1),
          update: rustFeedUpdate(revision: 1, posts: [loaded]),
        ),
        (
          at: const Duration(seconds: 2),
          update: rustFeedUpdate(revision: 2, posts: [loaded]),
        ),
      ])
        ..moreAvailable = true;
      final source = RustFeedRemoteSource(port: port);
      List<VideoPost>? result;

      source
          .loadRemoteFeed(searchQuery: 'ghost', olderThan: cursor)
          .then((posts) => result = posts);
      async.elapse(const Duration(seconds: 3));

      expect(result, isEmpty, reason: 'nothing at or past the cursor');
      expect(
        port.closedFeedIds,
        isEmpty,
        reason: 'the feed stays open for the pulls after this one',
      );
    });
  });
}
