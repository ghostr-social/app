import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';

import '../support/live_video_search_updates.dart';
import '../support/paged_search_repository.dart';
import '../support/sample_data.dart';

void main() {
  final updates = LiveVideoSearchUpdates();
  final repository = PagedSearchRepository(pages: [
    [samplePost(id: 'early')],
  ]);

  blocTest<SearchCubit, SearchState>(
    'matching posts accumulate from passive Rust updates',
    build: () => SearchCubit(repository, updates: updates),
    act: (cubit) async {
      await cubit.search('ghost');
      updates.publish(
        'ghost',
        VideoFeedPage(posts: [
          samplePost(id: 'early'),
          samplePost(id: 'later'),
        ]),
      );
      await pumpEventQueue();
    },
    expect: () => [
      isA<SearchLoading>(),
      isA<SearchLoaded>().having(_ids, 'ids', ['early']),
      isA<SearchLoaded>().having(_ids, 'ids', ['early', 'later']),
    ],
    verify: (_) => expect(repository.queries, ['ghost']),
  );
}

List<String> _ids(SearchLoaded state) {
  return state.videos.map((video) => video.id.value).toList();
}
