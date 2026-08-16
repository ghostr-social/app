import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('excludes watched videos on load but not session refresh', () async {
    final repository = FakeVideoCatalogRepository(forYouFeed: [samplePost()]);
    final cubit = FeedCubit(
      FeedDependencies(feed: repository, engagement: repository),
    );
    addTearDown(cubit.close);

    await cubit.load();
    expect(repository.loadFeedExclusions, [true]);

    await cubit.refresh();
    expect(repository.loadFeedExclusions, [true, false]);
  });
}
