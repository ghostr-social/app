import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';

import '../support/paged_search_repository.dart';
import '../support/sample_data.dart';

void main() {
  test('older search pages append without duplicates until exhausted',
      () async {
    final repository = PagedSearchRepository(pages: [
      [samplePost(id: 'a', publishedAt: DateTime.utc(2026, 3, 2))],
      [
        samplePost(id: 'a', publishedAt: DateTime.utc(2026, 3, 2)),
        samplePost(id: 'b', publishedAt: DateTime.utc(2026, 3, 1)),
      ],
    ]);
    final cubit = SearchCubit(repository);
    addTearDown(cubit.close);

    await cubit.search('ghost');
    var state = cubit.state as SearchLoaded;
    expect(state.hasMore, isTrue);

    await cubit.loadMore();
    state = cubit.state as SearchLoaded;
    expect(state.videos.map((video) => video.id.value), ['a', 'b']);
    expect(state.hasMore, isFalse);
    expect(state.isLoadingMore, isFalse);
    expect(repository.olderThans.last, isNotNull);

    await cubit.loadMore();
    expect(repository.queries, hasLength(2));
  });
}
