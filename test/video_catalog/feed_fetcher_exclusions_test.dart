import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_fetcher.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('watched videos stay excluded while hunting and resyncing', () async {
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [samplePost(id: 'post-0')],
    );
    final fetcher = FeedFetcher(repository);

    final fresh = await fetcher.unwatched(FeedKind.forYou);
    await fetcher.resync(FeedKind.forYou);

    expect(repository.loadFeedExclusions, [true, true]);
    expect((fresh as FeedFetched).posts.single.id.value, 'post-0');
  });
}
