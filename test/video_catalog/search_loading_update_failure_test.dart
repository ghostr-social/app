import 'dart:async';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_repository.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';

import '../support/live_video_search_updates.dart';

void main() {
  final repository = _PendingSearchRepository();
  final updates = LiveVideoSearchUpdates();

  blocTest<SearchCubit, SearchState>(
    'a live-update failure survives a later empty initial response',
    build: () => SearchCubit(repository, updates: updates),
    act: (cubit) async {
      final search = cubit.search('ghost');
      await pumpEventQueue();
      updates.fail('ghost', StateError('native watcher stopped'));
      await pumpEventQueue();
      repository.completeEmpty();
      await search;
    },
    expect: () => [
      isA<SearchLoading>(),
      isA<SearchFailure>().having(
        (state) => state.message,
        'message',
        'Live search updates paused.',
      ),
    ],
    tearDown: updates.close,
  );
}

final class _PendingSearchRepository implements VideoSearchRepository {
  final Completer<VideoFeedPage> _videos = Completer<VideoFeedPage>();

  void completeEmpty() {
    _videos.complete(VideoFeedPage(posts: const []));
  }

  @override
  Future<VideoFeedPage> searchVideos(String query, {DateTime? olderThan}) {
    return _videos.future;
  }

  @override
  Future<VideoFeedPage> loadMoreVideos(String query) => _videos.future;

  @override
  Future<List<ProfileSummary>> searchCreators(String query) async => const [];
}
