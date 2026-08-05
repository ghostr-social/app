import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

import '../support/fake_rust_feed_port.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  // A failed primary query raises AppFailure so the feed can show its
  // error state; an empty list would read as "nothing was posted".
  test('raises the shared feed failure when the page fails', () async {
    final port = FakeRustFeedPort(updates: [
      rustFeedBaseline(),
      rustFeedUpdate(revision: 1, stage: FfiFeedStage.failed),
    ]);
    final source = RustFeedRemoteSource(port: port);

    await expectLater(
      source.loadRemoteFeed(searchQuery: 'ghost'),
      throwsA(isA<AppFailure>().having(
        (failure) => failure.message,
        'message',
        'Could not load Nostr videos.',
      )),
    );
    expect(port.closedFeedIds, [port.feedId]);
  });
}
