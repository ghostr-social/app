import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';

import '../support/live_video_search_updates.dart';
import '../support/paged_search_repository.dart';
import '../support/sample_data.dart';

void main() {
  final updates = LiveVideoSearchUpdates();

  blocTest<SearchCubit, SearchState>(
    'a settled full snapshot removes a provisional video',
    build: () => SearchCubit(
      PagedSearchRepository(pages: [
        [samplePost(id: 'provisional'), samplePost(id: 'kept')],
      ]),
      updates: updates,
    ),
    act: (cubit) async {
      await cubit.search('ghost');
      updates.publish(
        'ghost',
        VideoFeedPage(posts: [samplePost(id: 'kept')]),
      );
      await pumpEventQueue();
    },
    expect: () => [
      isA<SearchLoading>(),
      isA<SearchLoaded>().having(
        (state) => _ids(state),
        'initial ids',
        ['provisional', 'kept'],
      ),
      isA<SearchLoaded>().having(
        (state) => _ids(state),
        'settled ids',
        ['kept'],
      ),
    ],
  );
}

List<String> _ids(SearchLoaded state) {
  return state.videos.map((video) => video.id.value).toList();
}
