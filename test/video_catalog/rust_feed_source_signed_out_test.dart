import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';

import '../support/fake_rust_feed_port.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  // ndk parity: signed out the main feed is still an unscoped relay
  // query (ndk_nostr_outbox_directory.dart falls back to the bootstrap
  // relays) and never fails. Rust scopes its main feed to a signed-in
  // key, so the closest non-failing behavior is an empty page — see the
  // viewer-less main feed blocker in the plan §5 notes.
  test('serves an empty main feed while signed out instead of failing',
      () async {
    final port = FakeRustFeedPort(updates: [
      rustFeedUpdate(revision: 1, posts: [rustFeedPost()]),
    ]);
    final source = RustFeedRemoteSource(port: port);

    final posts = await source.loadRemoteFeed();

    expect(posts, isEmpty);
    expect(port.openedSpecs, isEmpty);
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
