import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_search_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';

import '../support/fakes.dart';
import '../support/live_video_search_updates.dart';
import '../support/paged_search_repository.dart';
import '../support/sample_data.dart';

void main() {
  test('search pagination excludes watched videos', () async {
    final watched = samplePost(id: 'watched');
    final fresh = samplePost(id: 'fresh');
    final source = PagedSearchRepository(
      pages: [
        [watched, fresh],
      ],
    );
    final search = WatchAwareVideoSearchRepository(
      search: source,
      updates: LiveVideoSearchUpdates(),
      history: FakeWatchHistoryRepository(
        entries: [
          WatchHistoryEntry.fromPost(watched, DateTime.utc(2026, 8, 15)),
        ],
      ),
      failureReporter: RecordingFailureReporter(),
    );

    final page = await search.loadMoreVideos('clip');

    expect(page.posts.map((post) => post.id.value), ['fresh']);
    expect(source.loadMoreQueries, ['clip']);
  });
}
