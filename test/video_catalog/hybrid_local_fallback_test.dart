import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';

import '../support/fakes.dart';
import '../support/hybrid_repository_harness.dart';
import '../support/sample_data.dart';

void main() {
  test('returns published local videos when the remote feed fails', () async {
    final remote = FakeRemoteVideoSource([])
      ..failure = const AppFailure('relays unavailable');
    final harness = await buildHybridRepositoryHarness(remote);
    final localPost = samplePost(id: 'local');
    await harness.localStore.savePublishedPosts([localPost]);

    final posts = await harness.feed.loadFeed(FeedKind.forYou);

    expect(posts.map((post) => post.id), ['local']);
    expect(
      harness.failureReporter.sources,
      contains('HybridVideoReader.load'),
    );
  });
}
