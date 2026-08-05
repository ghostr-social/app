import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';

import '../support/paged_search_repository.dart';
import '../support/sample_data.dart';

void main() {
  late PagedSearchRepository repository;

  blocTest<SearchCubit, SearchState>(
    'a failed older page keeps the visible search retryable',
    build: () {
      repository = PagedSearchRepository(pages: [
        [samplePost(id: 'visible')],
        [samplePost(id: 'older')],
      ]);
      return SearchCubit(repository);
    },
    act: (cubit) async {
      await cubit.search('ghost');
      repository.videosFailure = StateError('relay unavailable');
      await cubit.loadMore();
    },
    expect: () => [
      isA<SearchLoading>(),
      isA<SearchLoaded>(),
      isA<SearchLoaded>().having(
        (state) => state.isLoadingMore,
        'loading older',
        isTrue,
      ),
      isA<SearchLoaded>()
          .having((state) => state.videos.single.id.value, 'video', 'visible')
          .having((state) => state.isLoadingMore, 'loading older', isFalse)
          .having((state) => state.hasMore, 'has more', isTrue)
          .having(
            (state) => state.notice,
            'notice',
            'Older search results are unavailable right now.',
          ),
    ],
  );
}
