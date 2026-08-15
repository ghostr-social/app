import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a creator profile remains an explicit replay surface', () async {
    final viewer = sampleCreator(id: 'viewer');
    final creator = sampleCreator(id: 'creator');
    final posts = [
      samplePost(id: 'first', creator: creator),
      samplePost(id: 'second', creator: creator),
    ];
    final catalog = FakeVideoCatalogRepository(
      forYouFeed: const [],
      feed: FakeFeedScenario(
        profiles: {
          creator.id: sampleProfileDetails(profile: creator, posts: posts),
        },
      ),
    );
    final cubit = AppControllerFactory(
      buildFakeDependencies(catalogRepository: catalog),
    ).profileFeed(viewer, posts.first);
    addTearDown(cubit.close);
    await cubit.load();

    cubit.pageChanged(1);
    await pumpEventQueue();

    final loaded = cubit.state as FeedLoaded;
    expect(loaded.posts.map((post) => post.id.value), ['first', 'second']);
    expect(loaded.activeIndex, 1);
  });
}
