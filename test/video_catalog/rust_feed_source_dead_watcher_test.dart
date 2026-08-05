import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';

import '../support/fake_rust_feed_port.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  test('reports a native watcher that ends while search is active', () async {
    final source = RustFeedRemoteSource(
      port: FakeRustFeedPort(updates: [rustFeedBaseline()]),
    );

    await expectLater(
      source.watchRemoteFeed(searchQuery: 'ghost'),
      emitsError(isA<AppFailure>()),
    );
  });
}
