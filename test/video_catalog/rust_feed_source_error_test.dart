import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';

import '../support/fake_rust_feed_port.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  // ndk parity: NdkNostrVideoEventQuery surfaces every transport
  // problem as AppFailure('Could not load Nostr videos.').
  test('translates an open failure into the shared feed failure', () async {
    final port = FakeRustFeedPort()..openError = StateError('engine down');
    final source = RustFeedRemoteSource(port: port);

    await expectLater(
      source.loadRemoteFeed(searchQuery: 'ghost'),
      throwsA(isA<AppFailure>().having(
        (failure) => failure.message,
        'message',
        'Could not load Nostr videos.',
      )),
    );
    expect(port.closedFeedIds, isEmpty);
  });

  test('translates a stream failure and still closes the feed', () async {
    final port = FakeRustFeedPort(updates: [rustFeedBaseline()])
      ..streamError = StateError('watcher died');
    final source = RustFeedRemoteSource(port: port);

    await expectLater(
      source.loadRemoteFeed(searchQuery: 'ghost'),
      throwsA(isA<AppFailure>()),
    );
    expect(port.closedFeedIds, [port.feedId]);
  });
}
