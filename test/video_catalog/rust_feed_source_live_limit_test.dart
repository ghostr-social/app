import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_sessions.dart';

import '../support/live_rust_feed_port.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  // Feeds that live as long as the app must not pile up as long as the
  // app: each holds rows in Rust and one snapshot stream, so a source
  // keeps only the feeds a viewer moves between.
  test('keeps only the last few feeds open, closing the oldest', () async {
    final port = LiveRustFeedPort(
      firstPage: [
        rustFeedUpdate(revision: 1, posts: [rustFeedPost()]),
      ],
    );
    final source = RustFeedRemoteSource(port: port);

    for (var index = 0; index <= rustFeedLiveLimit; index++) {
      await source.loadRemoteFeed(searchQuery: 'query-$index');
    }

    expect(port.openedSpecs, hasLength(rustFeedLiveLimit + 1));
    expect(
      port.closedFeedIds.map((id) => id.value),
      ['1'],
      reason: 'the least recently pulled feed is the one to drop',
    );
  });

  test('reopens nothing while a feed is still the most recently pulled',
      () async {
    final port = LiveRustFeedPort(
      firstPage: [
        rustFeedUpdate(revision: 1, posts: [rustFeedPost()]),
      ],
    );
    final source = RustFeedRemoteSource(port: port);

    await source.loadRemoteFeed(searchQuery: 'kept');
    for (var index = 1; index < rustFeedLiveLimit; index++) {
      await source.loadRemoteFeed(searchQuery: 'query-$index');
      await source.loadRemoteFeed(searchQuery: 'kept');
    }
    await source.loadRemoteFeed(searchQuery: 'overflow');

    expect(port.closedFeedIds.map((id) => id.value), isNot(contains('1')));
  });
}
