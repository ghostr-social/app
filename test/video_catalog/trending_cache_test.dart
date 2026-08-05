import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/recent_videos_trending_hashtags.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/sample_data.dart';

void main() {
  test('trending is computed from recent videos and cached for a while',
      () async {
    var now = DateTime.utc(2026, 8, 1, 12);
    final source = _CountingSource();
    final trending = RecentVideosTrendingHashtags(
      source,
      clock: () => now,
    );

    expect(await trending.trendingHashtags(), ['dance']);
    expect(await trending.trendingHashtags(), ['dance']);
    expect(source.calls, 1);

    now = now.add(const Duration(minutes: 16));
    expect(await trending.trendingHashtags(), ['dance']);
    expect(source.calls, 2);
  });
}

class _CountingSource implements RemoteVideoSource {
  int calls = 0;

  @override
  Future<List<VideoPost>> loadRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
    DateTime? olderThan,
  }) async {
    calls += 1;
    return [
      samplePost(hashtags: const ['dance'])
    ];
  }

  @override
  Future<List<VideoPost>> loadMoreRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
  }) async {
    return const <VideoPost>[];
  }

  @override
  Stream<RemoteVideoSnapshot> watchRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
  }) {
    return const Stream.empty();
  }
}
