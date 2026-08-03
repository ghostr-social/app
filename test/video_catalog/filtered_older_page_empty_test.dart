import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';

import '../support/fake_remote_video_source.dart';
import '../support/hybrid_repository_harness.dart';
import '../support/sample_data.dart';

void main() {
  test('an empty older page marks the feed as exhausted', () async {
    final remote = FakeRemoteVideoSource([samplePost()]);
    final harness = await buildHybridRepositoryHarness(remote);

    final page = await harness.feed.loadOlderFeed(
      FeedKind.forYou,
      olderThan: DateTime.utc(2026, 8, 1),
    );

    expect(page.posts, isEmpty);
    expect(page.hasMore, isFalse);
    expect(page.nextOlderThan, isNull);
  });
}
