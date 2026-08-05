import 'dart:async';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_repository.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';

void main() {
  late _PendingVideos repository;

  blocTest<SearchCubit, SearchState>(
    'typing a new query immediately supersedes the in-flight result',
    build: () {
      repository = _PendingVideos();
      return SearchCubit(repository);
    },
    act: (cubit) async {
      final oldSearch = cubit.search('old');
      cubit.queryChanged('new');
      repository.video.complete(VideoFeedPage(posts: const []));
      await oldSearch;
    },
    expect: () => [
      isA<SearchLoading>().having((state) => state.query, 'query', 'old'),
    ],
  );
}

class _PendingVideos implements VideoSearchRepository {
  final video = Completer<VideoFeedPage>();

  @override
  Future<VideoFeedPage> loadMoreVideos(String query) => video.future;

  @override
  Future<VideoFeedPage> searchVideos(String query, {DateTime? olderThan}) {
    return video.future;
  }

  @override
  Future<List<ProfileSummary>> searchCreators(String query) async {
    return const <ProfileSummary>[];
  }
}
