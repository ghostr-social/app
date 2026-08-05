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
    'a settled empty snapshot removes all provisional videos',
    build: () => SearchCubit(
      PagedSearchRepository(pages: [
        [samplePost(id: 'provisional')],
      ]),
      updates: updates,
    ),
    act: (cubit) async {
      await cubit.search('ghost');
      updates.publish('ghost', VideoFeedPage(posts: const []));
      await pumpEventQueue();
    },
    expect: () => [
      isA<SearchLoading>(),
      isA<SearchLoaded>(),
      isA<SearchEmpty>().having((state) => state.query, 'query', 'ghost'),
    ],
  );
}
