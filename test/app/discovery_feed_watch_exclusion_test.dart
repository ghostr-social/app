import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('search never serves watched videos', () async {
    final watched = samplePost(id: 'watched');
    final fresh = samplePost(id: 'fresh');
    final history = FakeWatchHistoryRepository(
      entries: [WatchHistoryEntry.fromPost(watched, DateTime.utc(2026, 8, 15))],
    );
    final catalog = FakeVideoCatalogRepository(
      forYouFeed: const [],
      feed: FakeFeedScenario(searchResults: [watched, fresh]),
    );
    final dependencies = buildFakeDependencies(
      catalogRepository: catalog,
      overrides: FakeDependencyOverrides(watchHistory: history),
    );
    final cubit = AppControllerFactory(dependencies).discoveryFeed('clip');
    addTearDown(cubit.close);

    await cubit.load();

    final loaded = cubit.state as FeedLoaded;
    expect(loaded.posts.map((post) => post.id.value), ['fresh']);
  });
}
