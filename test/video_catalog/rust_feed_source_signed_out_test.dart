import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';

import '../support/fake_rust_feed_port.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  // ndk parity: signed out the main feed is still an unscoped relay
  // query (ndk_nostr_outbox_directory.dart knows no follows without an
  // account and falls back to the bootstrap relays), so it serves a
  // global recent page instead of nothing. Rust names no viewer on that
  // feed and skips mute filtering (discovery/feed_spec.rs).
  test('serves the global main feed while signed out', () async {
    final port = FakeRustFeedPort(updates: [
      rustFeedUpdate(revision: 1, posts: [rustFeedPost()]),
    ]);
    final source = RustFeedRemoteSource(port: port);

    final posts = await source.loadRemoteFeed();

    expect(posts, hasLength(1));
    expect(port.openedSpecs.single.kind, 'main');
    expect(port.openedSpecs.single.viewerPubkey, isNull);
  });

  test('still opens the query feeds a signed-out viewer can read', () async {
    final port = FakeRustFeedPort(updates: [
      rustFeedUpdate(revision: 1, posts: [rustFeedPost()]),
    ]);
    final source = RustFeedRemoteSource(port: port);

    final posts = await source.loadRemoteFeed(searchQuery: 'ghost');

    expect(posts, hasLength(1));
    expect(port.openedSpecs.single.kind, 'search');
  });
}
