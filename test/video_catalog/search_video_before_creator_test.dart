import 'dart:async';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_repository.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';

import '../support/sample_data.dart';

void main() {
  late _SlowCreators repository;

  blocTest<SearchCubit, SearchState>(
    'ready videos do not wait for a slow creator lookup',
    build: () {
      repository = _SlowCreators();
      return SearchCubit(repository);
    },
    act: (cubit) async {
      final search = cubit.search('ghost');
      await pumpEventQueue();
      repository.creators.complete([sampleCreator(displayName: 'Alice')]);
      await search;
    },
    expect: () => [
      isA<SearchLoading>(),
      isA<SearchLoaded>()
          .having((state) => state.videos, 'videos', hasLength(1))
          .having((state) => state.creators, 'creators', isEmpty),
      isA<SearchLoaded>().having(
        (state) => state.creators.single.displayName,
        'creator',
        'Alice',
      ),
    ],
  );
}

class _SlowCreators implements VideoSearchRepository {
  final creators = Completer<List<ProfileSummary>>();

  @override
  Future<VideoFeedPage> loadMoreVideos(String query) => searchVideos(query);

  @override
  Future<VideoFeedPage> searchVideos(String query,
      {DateTime? olderThan}) async {
    return VideoFeedPage(posts: [samplePost()]);
  }

  @override
  Future<List<ProfileSummary>> searchCreators(String query) => creators.future;
}
