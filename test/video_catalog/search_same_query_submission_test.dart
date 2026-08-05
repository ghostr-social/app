import 'dart:async';

import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_repository.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';

void main() {
  test('submitting an auto-started query does not request it twice', () {
    fakeAsync((async) {
      final repository = _PendingSearch();
      final cubit = SearchCubit(repository);

      cubit.queryChanged('bitcoin');
      async.elapse(const Duration(milliseconds: 300));
      async.flushMicrotasks();
      expect(repository.videoRequests, 1);

      unawaited(cubit.search('bitcoin'));
      async.flushMicrotasks();

      expect(repository.videoRequests, 1);
      repository.result.complete(VideoFeedPage(posts: const []));
      async.flushMicrotasks();
      unawaited(cubit.close());
    });
  });
}

final class _PendingSearch implements VideoSearchRepository {
  final result = Completer<VideoFeedPage>();
  var videoRequests = 0;

  @override
  Future<VideoFeedPage> searchVideos(String query, {DateTime? olderThan}) {
    videoRequests += 1;
    return result.future;
  }

  @override
  Future<List<ProfileSummary>> searchCreators(String query) async => const [];

  @override
  Future<VideoFeedPage> loadMoreVideos(String query) => result.future;
}
