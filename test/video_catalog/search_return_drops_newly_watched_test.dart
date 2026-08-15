import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_search_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('returning to search removes a video watched while away', () async {
    final watchedElsewhere = samplePost(id: 'watched-elsewhere');
    final fresh = samplePost(id: 'fresh');
    final history = FakeWatchHistoryRepository();
    final source = FakeVideoCatalogRepository(
      forYouFeed: const [],
      feed: FakeFeedScenario(searchResults: [watchedElsewhere, fresh]),
    );
    final repository = WatchAwareVideoSearchRepository(
      search: source,
      updates: source,
      history: history,
      failureReporter: RecordingFailureReporter(),
    );
    final cubit = SearchCubit(repository, updates: repository);
    addTearDown(cubit.close);
    await cubit.search('clip');
    expect((cubit.state as SearchLoaded).videos, hasLength(2));

    cubit.deactivate();
    await history.record(
      WatchHistoryEntry.fromPost(watchedElsewhere, DateTime.utc(2026, 8, 15)),
    );
    await cubit.refresh();

    final returned = cubit.state as SearchLoaded;
    expect(returned.videos.map((post) => post.id.value), ['fresh']);
  });
}
