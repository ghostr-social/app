import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';

import '../support/live_rust_feed_port.dart';
import '../support/nostr_test_values.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  test('a new native session replaces warm feeds for the same viewer',
      () async {
    final viewer = NostrPublicKeyHex.parse(testViewerPublicKey);
    final port = LiveRustFeedPort(
      firstPage: [
        rustFeedUpdate(
          revision: 1,
          posts: [rustFeedPost(caption: 'old session')],
        ),
      ],
    );
    final source = RustFeedRemoteSource(port: port, viewer: () => viewer);
    expect((await source.loadRemoteFeed()).single.caption, 'old session');

    port
      ..sessionGeneration = BigInt.one
      ..firstPage = [
        rustFeedUpdate(
          revision: 1,
          posts: [rustFeedPost(caption: 'new session')],
        ),
      ];

    expect((await source.loadRemoteFeed()).single.caption, 'new session');
    expect(port.closedFeedIds.map((feed) => feed.value), ['1']);
    expect(port.openedSpecs, hasLength(2));
  });
}
