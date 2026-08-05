import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_updates.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';

import '../support/live_video_search_updates.dart';
import '../support/paged_search_repository.dart';
import '../support/sample_data.dart';

void main() {
  final updates = LiveVideoSearchUpdates();
  final repository = PagedSearchRepository(pages: [
    [samplePost(id: 'head')],
    [samplePost(id: 'older')],
  ]);

  blocTest<SearchCubit, SearchState>(
    'pagination waits for the native first page to settle',
    build: () => SearchCubit(repository, updates: updates),
    act: (cubit) async {
      await cubit.search('ghost');
      await cubit.loadMore();
      updates.publish(
        'ghost',
        VideoFeedPage(
          posts: [samplePost(id: 'head')],
          nextOlderThan: DateTime.utc(2025),
        ),
        phase: VideoSearchPhase.loading,
      );
      await pumpEventQueue();
      await cubit.loadMore();
      updates.publish(
        'ghost',
        VideoFeedPage(
          posts: [samplePost(id: 'head')],
          nextOlderThan: DateTime.utc(2025),
        ),
      );
      await pumpEventQueue();
      await cubit.loadMore();
    },
    expect: () => [
      isA<SearchLoading>(),
      _loaded(canLoadMore: false, loading: false),
      _loaded(canLoadMore: false, loading: false),
      _loaded(canLoadMore: true, loading: false),
      _loaded(canLoadMore: true, loading: true),
      _loaded(canLoadMore: false, loading: false),
    ],
    verify: (_) => expect(repository.loadMoreQueries, ['ghost']),
  );
}

Matcher _loaded({required bool canLoadMore, required bool loading}) {
  return isA<SearchLoaded>()
      .having((state) => state.canLoadMore, 'can load more', canLoadMore)
      .having((state) => state.isLoadingMore, 'loading more', loading);
}
