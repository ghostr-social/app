import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';

import '../support/live_rust_feed_port.dart';
import '../support/nostr_test_values.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  // Follows, mutes and outbox routing belong to the signed-in account
  // and colour every feed Rust assembles, not only the main one — so
  // signing in as somebody else drops all of them, search feeds too.
  test('closes every live feed when the signed-in account changes', () async {
    NostrPublicKeyHex? viewer = NostrPublicKeyHex.parse(testViewerPublicKey);
    final port = LiveRustFeedPort(
      firstPage: [
        rustFeedUpdate(revision: 1, posts: [rustFeedPost()]),
      ],
    );
    final source = RustFeedRemoteSource(port: port, viewer: () => viewer);

    await source.loadRemoteFeed();
    await source.loadRemoteFeed(searchQuery: 'ghost');
    viewer = NostrPublicKeyHex.parse(testCreatorPublicKey);
    await source.loadRemoteFeed();

    expect(port.closedFeedIds.map((id) => id.value), ['1', '2']);
    expect(port.openedSpecs.last.viewerPubkey, testCreatorPublicKey);
  });

  test('closes the account feeds when the viewer signs out', () async {
    NostrPublicKeyHex? viewer = NostrPublicKeyHex.parse(testViewerPublicKey);
    final port = LiveRustFeedPort(
      firstPage: [
        rustFeedUpdate(revision: 1, posts: [rustFeedPost()]),
      ],
    );
    final source = RustFeedRemoteSource(port: port, viewer: () => viewer);

    await source.loadRemoteFeed();
    viewer = null;
    await source.loadRemoteFeed();

    expect(port.closedFeedIds.map((id) => id.value), ['1']);
    expect(port.openedSpecs.last.viewerPubkey, isNull);
  });
}
