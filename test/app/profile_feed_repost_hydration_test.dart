import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fake_dependencies.dart';
import '../support/fake_video_catalog_scenarios.dart';
import '../support/repost_hydrating_catalog.dart';
import '../support/repost_samples.dart';
import '../support/sample_data.dart';

void main() {
  test('profile feed hydrates repost state before its first toggle', () async {
    final post = repostablePost();
    final viewer = sampleCreator(id: 'viewer');
    final catalog = RepostHydratingCatalog(
      forYouFeed: [post],
      feed: FakeFeedScenario(
        profiles: {
          post.creator.id: sampleProfileDetails(
            profile: post.creator,
            posts: [post],
          ),
        },
      ),
    );
    final factory = AppControllerFactory(
      buildFakeDependencies(catalogRepository: catalog),
    );
    final cubit = factory.profileFeed(viewer, post);
    addTearDown(cubit.close);

    await cubit.load();
    final hydrated = (cubit.state as FeedLoaded).posts.single;
    expect(hydrated.viewerHasReposted, isTrue);

    await cubit.toggleRepost(hydrated);
    expect(catalog.toggleInputs.single.viewerHasReposted, isTrue);
    expect((cubit.state as FeedLoaded).posts.single.viewerHasReposted, isFalse);
  });
}
