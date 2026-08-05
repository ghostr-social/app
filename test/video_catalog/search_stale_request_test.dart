import 'dart:async';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_repository.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';

import '../support/sample_data.dart';

void main() {
  late _PendingSearchRepository repository;

  blocTest<SearchCubit, SearchState>(
    'an older search completion cannot replace the latest query',
    build: () {
      repository = _PendingSearchRepository();
      return SearchCubit(repository);
    },
    act: (cubit) async {
      final older = cubit.search('old');
      await pumpEventQueue();
      final latest = cubit.search('new');
      await pumpEventQueue();
      repository.complete('new', [samplePost(id: 'new')]);
      await latest;
      repository.complete('old', [samplePost(id: 'old')]);
      await older;
    },
    expect: () => [
      isA<SearchLoading>().having((state) => state.query, 'query', 'old'),
      isA<SearchLoading>().having((state) => state.query, 'query', 'new'),
      isA<SearchLoaded>()
          .having((state) => state.query, 'query', 'new')
          .having((state) => state.videos.single.id.value, 'video', 'new'),
    ],
  );
}

class _PendingSearchRepository implements VideoSearchRepository {
  final pending = <String, Completer<List<VideoPost>>>{};

  @override
  Future<VideoFeedPage> loadMoreVideos(String query) => searchVideos(query);

  @override
  Future<VideoFeedPage> searchVideos(String query, {DateTime? olderThan}) {
    return pending
        .putIfAbsent(query, Completer.new)
        .future
        .then((posts) => VideoFeedPage(posts: posts));
  }

  @override
  Future<List<ProfileSummary>> searchCreators(String query) async {
    return const <ProfileSummary>[];
  }

  void complete(String query, List<VideoPost> posts) {
    pending[query]!.complete(posts);
  }
}
