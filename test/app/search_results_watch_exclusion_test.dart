import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('search result rows never list a watched video', () async {
    final watched = samplePost(id: 'watched');
    final fresh = samplePost(id: 'fresh');
    final history = FakeWatchHistoryRepository(
      entries: [WatchHistoryEntry.fromPost(watched, DateTime.utc(2026, 8, 15))],
    );
    final catalog = FakeVideoCatalogRepository(
      forYouFeed: const [],
      feed: FakeFeedScenario(searchResults: [watched, fresh]),
    );
    final cubit = AppControllerFactory(
      buildFakeDependencies(
        catalogRepository: catalog,
        overrides: FakeDependencyOverrides(watchHistory: history),
      ),
    ).search();
    addTearDown(cubit.close);

    await cubit.search('clip');

    final loaded = cubit.state as SearchLoaded;
    expect(loaded.videos.map((post) => post.id.value), ['fresh']);
  });
}
