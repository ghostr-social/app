import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fake_dependencies.dart';
import '../support/fake_video_catalog_scenarios.dart';
import '../support/repost_hydrating_catalog.dart';
import '../support/repost_samples.dart';

void main() {
  test('search feed hydrates repost state before its first toggle', () async {
    final post = repostablePost();
    final catalog = RepostHydratingCatalog(
      forYouFeed: [post],
      feed: FakeFeedScenario(searchResults: [post]),
    );
    final factory = AppControllerFactory(
      buildFakeDependencies(catalogRepository: catalog),
    );
    final cubit = factory.discoveryFeed(
      'clip',
      viewerId: ProfileId.parse('viewer'),
    );
    addTearDown(cubit.close);

    await cubit.load();
    final hydrated = (cubit.state as FeedLoaded).posts.single;
    expect(hydrated.viewerHasReposted, isTrue);

    await cubit.toggleRepost(hydrated);
    expect(catalog.toggleInputs.single.viewerHasReposted, isTrue);
    expect((cubit.state as FeedLoaded).posts.single.viewerHasReposted, isFalse);
  });
}
