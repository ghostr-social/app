import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';

import '../support/live_video_search_updates.dart';
import '../support/paged_search_repository.dart';
import '../support/sample_data.dart';

void main() {
  final updates = LiveVideoSearchUpdates();
  final visible = samplePost(id: 'visible');

  blocTest<SearchCubit, SearchState>(
    'a live-update failure reports a notice without hiding visible rows',
    build: () => SearchCubit(
      PagedSearchRepository(pages: [
        [visible],
      ]),
      updates: updates,
    ),
    act: (cubit) async {
      await cubit.search('ghost');
      updates.fail('ghost', StateError('native watcher stopped'));
      await pumpEventQueue();
    },
    expect: () => [
      isA<SearchLoading>(),
      isA<SearchLoaded>(),
      isA<SearchLoaded>()
          .having((state) => state.videos, 'videos', [visible]).having(
        (state) => state.notice,
        'notice',
        'Live search updates paused.',
      ),
    ],
    tearDown: updates.close,
  );
}
