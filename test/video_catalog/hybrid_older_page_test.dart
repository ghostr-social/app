import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';

import '../support/fake_remote_video_source.dart';
import '../support/hybrid_repository_harness.dart';
import '../support/sample_data.dart';

void main() {
  test('an older page is fetched with its cutoff and hydrated', () async {
    final remote = FakeRemoteVideoSource([samplePost(id: 'post-newest')])
      ..olderPosts = [samplePost(id: 'post-older')];
    final harness = await buildHybridRepositoryHarness(remote);
    final olderThan = DateTime.utc(2026, 8, 1);

    final page = await harness.feed.loadOlderFeed(
      FeedKind.forYou,
      olderThan: olderThan,
    );

    expect(remote.requestedOlderThan, [olderThan]);
    expect(page.posts.map((post) => post.id.value), ['post-older']);
    expect(page.hasMore, isTrue);
  });
}
