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
  final visible = samplePost(id: 'visible');

  blocTest<SearchCubit, SearchState>(
    'a failed empty native page keeps the current search retryable',
    build: () => SearchCubit(
      PagedSearchRepository(pages: [
        [visible],
      ]),
      updates: updates,
    ),
    act: (cubit) async {
      await cubit.search('ghost');
      updates.publish(
        'ghost',
        VideoFeedPage(posts: const []),
        phase: VideoSearchPhase.failed,
      );
      await pumpEventQueue();
    },
    expect: () => [
      isA<SearchLoading>(),
      isA<SearchLoaded>(),
      isA<SearchLoaded>()
          .having((state) => state.videos, 'videos', [visible]).having(
        (state) => state.notice,
        'notice',
        'Search relays are retrying.',
      ),
    ],
  );
}
