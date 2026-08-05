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
      postId: 'post-$index',
      eventId: publishedEventId(index + 1),
    ),
  );
}

void main() {
  test('returns progressive rows while Rust keeps searching', () {
    fakeAsync((async) {
      final port = TimedRustFeedPort([
        (
          at: const Duration(seconds: 1),
          update: rustFeedUpdate(
            revision: 1,
            stage: FfiFeedStage.loading,
            posts: _posts(2),
          ),
        ),
        (
          at: const Duration(seconds: 2),
          update: rustFeedUpdate(revision: 2, posts: _posts(6)),
        ),
      ]);
      final source = RustFeedRemoteSource(port: port);
      List<VideoPost>? result;

      source
          .loadRemoteFeed(searchQuery: 'ghost')
          .then((posts) => result = posts);
      async.elapse(const Duration(milliseconds: 1500));

      expect(result, hasLength(2));
    });
  });

  test('answers as soon as the page settles, long before the deadline', () {
    fakeAsync((async) {
      final port = TimedRustFeedPort([
        (
          at: const Duration(seconds: 1),
          update: rustFeedUpdate(revision: 1, posts: _posts(2)),
        ),
      ]);
      final source = RustFeedRemoteSource(port: port);
      List<VideoPost>? result;

      source
          .loadRemoteFeed(searchQuery: 'ghost')
          .then((posts) => result = posts);
      async.elapse(const Duration(seconds: 2));

      expect(result, hasLength(2));
    });
  });
}
