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
    'an initially empty search consumes a later Rust snapshot',
    build: () => SearchCubit(
      PagedSearchRepository(pages: const [<Never>[]]),
      updates: updates,
    ),
    act: (cubit) async {
      await cubit.search('ghost');
      updates.publish(
        'ghost',
        VideoFeedPage(posts: [samplePost(id: 'late')]),
      );
      await pumpEventQueue();
    },
    expect: () => [
      isA<SearchLoading>(),
      isA<SearchEmpty>(),
      isA<SearchLoaded>().having(
        (state) => state.videos.single.id.value,
        'post id',
        'late',
      ),
    ],
  );
}
