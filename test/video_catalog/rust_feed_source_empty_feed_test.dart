import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';

import '../support/fake_rust_feed_port.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  test('reports a dead feed instead of misclassifying it as empty', () async {
    final port = FakeRustFeedPort(updates: [rustFeedBaseline()]);
    final source = RustFeedRemoteSource(port: port);

    await expectLater(
      source.loadRemoteFeed(searchQuery: 'ghost'),
      throwsA(isA<AppFailure>()),
    );
    expect(port.closedFeedIds, [port.feedId]);
  });
}
