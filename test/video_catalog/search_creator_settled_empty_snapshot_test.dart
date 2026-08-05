import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';

import '../support/live_video_search_updates.dart';
import '../support/paged_search_repository.dart';
import '../support/sample_data.dart';

void main() {
  final updates = LiveVideoSearchUpdates();
  final creator = sampleCreator(displayName: 'Alice');

  blocTest<SearchCubit, SearchState>(
    'a settled empty video page preserves creator matches',
    build: () => SearchCubit(
      PagedSearchRepository(creators: [creator]),
      updates: updates,
    ),
    act: (cubit) async {
      await cubit.search('alice');
      updates.publish('alice', VideoFeedPage(posts: const []));
      await pumpEventQueue();
    },
    expect: () => [
      isA<SearchLoading>(),
      isA<SearchEmpty>(),
      isA<SearchLoaded>().having(
        (state) => state.creators,
        'creators',
        [creator],
      ),
      isA<SearchLoaded>().having(
        (state) => state.creators,
        'creators',
        [creator],
      ),
    ],
  );
}
