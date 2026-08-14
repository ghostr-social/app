import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

import '../support/nostr_test_values.dart';
import '../support/rust_feed_fixtures.dart';
import '../support/timed_rust_feed_port.dart';

List<FfiFeedPost> _posts(int count) {
  return List<FfiFeedPost>.generate(
    count,
    (index) => rustFeedPost(
      eventId: publishedEventId(index + 1),
      details: RustFeedPostDetails(postId: 'post-$index'),
    ),
  );
}

void main() {
  // A discovery plan waits up to eight seconds for its NIP-50 queries
  // (rust/src/discovery/search_queries.rs) and publishes the page as one
  // revision when the whole plan resolves, so the adapter's deadline has
  // to outlive the pipeline it reads.
  test('serves a page that lands after the old six-second deadline', () {
    fakeAsync((async) {
      final port = TimedRustFeedPort([
        (
          at: const Duration(seconds: 7),
          update: rustFeedUpdate(revision: 1, posts: _posts(6)),
        ),
      ]);
      final source = RustFeedRemoteSource(port: port);
      List<VideoPost>? result;

      source
          .loadRemoteFeed(searchQuery: 'ghost')
          .then((posts) => result = posts);
      async.elapse(const Duration(seconds: 20));

      expect(result, hasLength(6));
      expect(
        port.closedFeedIds,
        isEmpty,
        reason: 'the feed stays open for the pulls after this one',
      );
    });
  });
}
