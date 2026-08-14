import 'package:ghostr/features/reposts/domain/video_repost_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_repost_context.dart';

import 'fake_video_catalog_repository.dart';
import 'fake_video_catalog_scenarios.dart';

final class RepostHydratingCatalog extends FakeVideoCatalogRepository {
  RepostHydratingCatalog({
    required super.forYouFeed,
    super.feed = const FakeFeedScenario(),
  });

  final toggleInputs = <VideoPost>[];
  var hydratedBatches = 0;

  @override
  Future<List<VideoPost>> hydrateAll(
    List<VideoPost> posts, {
    VideoRepostHydration mode = VideoRepostHydration.prompt,
  }) async {
    hydratedBatches += 1;
    return posts
        .map(
          (post) => post.withRepost(
            true,
            observation: VideoRepostObservation.observed,
          ),
        )
        .toList(growable: false);
  }

  @override
  Future<VideoPost> toggleRepost(VideoPost post) async {
    toggleInputs.add(post);
    return post.withRepost(false, observation: VideoRepostObservation.observed);
  }
}
