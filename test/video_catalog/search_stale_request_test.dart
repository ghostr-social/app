import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_repository.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';

import '../support/sample_data.dart';

void main() {
  test('an older search completion cannot replace the latest query', () async {
    final repository = _PendingSearchRepository();
    final cubit = SearchCubit(repository);

    final older = cubit.search('old');
    final latest = cubit.search('new');
    repository.complete('new', [samplePost(id: 'new')]);
    await latest;
    repository.complete('old', [samplePost(id: 'old')]);
    await older;

    final state = cubit.state as SearchLoaded;
    expect(state.query, 'new');
    expect(state.videos.single.id.value, 'new');
    await cubit.close();
  });
}

class _PendingSearchRepository implements VideoSearchRepository {
  final pending = <String, Completer<List<VideoPost>>>{};

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
