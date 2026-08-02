import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';

import '../support/fakes.dart';
import '../support/hybrid_repository_harness.dart';

void main() {
  test('surfaces a remote feed failure when no local videos exist', () async {
    final remote = FakeRemoteVideoSource([])
      ..failure = const AppFailure('relays unavailable');
    final harness = await buildHybridRepositoryHarness(remote);

    await expectLater(
      harness.feed.loadFeed(FeedKind.forYou),
      throwsA(isA<AppFailure>()),
    );
  });
}
