import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_repository.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';

import '../support/sample_data.dart';

void main() {
  blocTest<SearchCubit, SearchState>(
    'a creator lookup failure does not hide ready videos',
    build: () => SearchCubit(_FailingCreators()),
    act: (cubit) => cubit.search('ghost'),
    expect: () => [
      isA<SearchLoading>().having((state) => state.query, 'query', 'ghost'),
      isA<SearchLoaded>()
          .having((state) => state.videos.single.id.value, 'video', 'post-1')
          .having((state) => state.creators, 'creators', isEmpty),
    ],
  );
}

class _FailingCreators implements VideoSearchRepository {
  @override
  Future<VideoFeedPage> loadMoreVideos(String query) => searchVideos(query);

  @override
  Future<VideoFeedPage> searchVideos(String query,
      {DateTime? olderThan}) async {
    return VideoFeedPage(posts: [samplePost()]);
  }

  @override
  Future<List<ProfileSummary>> searchCreators(String query) async {
    throw StateError('creator relay unavailable');
  }
}
