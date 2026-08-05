import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

import '../support/fake_rust_feed_port.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  // A signed-out main feed has no viewer-scoped routing or mute graph,
  // so Rust serves the configured relays' global recent page.
  test('serves the global main feed while signed out', () async {
    final port = FakeRustFeedPort(updates: [
      rustFeedUpdate(revision: 1, posts: [rustFeedPost()]),
    ]);
    final source = RustFeedRemoteSource(port: port);

    final posts = await source.loadRemoteFeed();

    expect(posts, hasLength(1));
    expect(port.openedSpecs.single.kind, FfiFeedKind.main);
    expect(port.openedSpecs.single.viewerPubkey, isNull);
  });

  test('still opens the query feeds a signed-out viewer can read', () async {
    final port = FakeRustFeedPort(updates: [
      rustFeedUpdate(revision: 1, posts: [rustFeedPost()]),
    ]);
    final source = RustFeedRemoteSource(port: port);

    final posts = await source.loadRemoteFeed(searchQuery: 'ghost');

    expect(posts, hasLength(1));
    expect(port.openedSpecs.single.kind, FfiFeedKind.search);
  });
}
