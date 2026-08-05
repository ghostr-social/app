import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_updates.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';

import '../support/live_video_search_updates.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'retry supersedes an active query after its live search fails',
    () async {
      final repository = _PendingSearchRepository();
      final updates = LiveVideoSearchUpdates();
      final cubit = SearchCubit(repository, updates: updates);

      final firstSearch = cubit.search('bitcoin');
      await pumpEventQueue();
      updates.publish(
        'bitcoin',
        VideoFeedPage(posts: const []),
        phase: VideoSearchPhase.failed,
      );
      await pumpEventQueue();
      expect(cubit.state, isA<SearchFailure>());

      final retry = cubit.retry();
      await pumpEventQueue();
      expect(repository.requests, hasLength(2));

      repository.requests.last.complete(VideoFeedPage(posts: [samplePost()]));
      await retry;
      expect(cubit.state, isA<SearchLoaded>());

      repository.requests.first.complete(VideoFeedPage(posts: const []));
      await firstSearch;
      await cubit.close();
      await updates.close();
    },
  );
}

final class _PendingSearchRepository implements VideoSearchRepository {
  final List<Completer<VideoFeedPage>> requests = [];

  @override
  Future<VideoFeedPage> searchVideos(String query, {DateTime? olderThan}) {
    final request = Completer<VideoFeedPage>();
    requests.add(request);
    return request.future;
  }

  @override
  Future<List<ProfileSummary>> searchCreators(String query) async => const [];

  @override
  Future<VideoFeedPage> loadMoreVideos(String query) => requests.last.future;
}
