import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';

import '../support/controllable_video_feed_updates.dart';
import '../support/fake_dependencies.dart';
import '../support/fake_video_catalog_repository.dart';
import '../support/sample_data.dart';

void main() {
  test('only the main feed subscribes to canonical Rust revisions', () async {
    final updates = ControllableVideoFeedUpdates();
    addTearDown(updates.close);
    final catalog = FakeVideoCatalogRepository(
      forYouFeed: [samplePost(id: 'main')],
    );
    final factory = AppControllerFactory(
      buildFakeDependencies(catalogRepository: catalog, feedUpdates: updates),
    );
    final main = factory.feed();
    final discovery = factory.discoveryFeed('ghost');
    addTearDown(main.close);
    addTearDown(discovery.close);

    await main.load();
    await discovery.load();

    expect(updates.watchedKinds, [FeedKind.forYou]);
  });
}
