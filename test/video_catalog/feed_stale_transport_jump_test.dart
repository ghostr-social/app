import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/controllable_video_delivery_updates.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a rescue jump cannot consume navigation in a new feed', () async {
    final initial = List.generate(3, (index) => samplePost(id: 'old-$index'));
    final fresh = List.generate(3, (index) => samplePost(id: 'new-$index'));
    final updates = ControllableVideoDeliveryUpdates();
    final repository = FakeVideoCatalogRepository(forYouFeed: initial);
    final cubit = FeedCubit(
      FeedDependencies(
        feed: repository,
        engagement: repository,
        optional: FeedOptionalDependencies(
          delivery: FeedDeliveryDependencies(deliveryUpdates: updates),
        ),
      ),
    );
    addTearDown(cubit.close);
    addTearDown(updates.close);
    await cubit.load();
    updates.publish(initial[1], startable: false);
    updates.publish(initial[2], startable: true);
    cubit.pageChanged(1);
    await pumpEventQueue();
    expect((cubit.state as FeedLoaded).activeIndex, 2);

    repository.forYouFeed
      ..clear()
      ..addAll(fresh);
    await cubit.reload();
    cubit.pageChanged(2);

    expect((cubit.state as FeedLoaded).activeIndex, 2);
  });
}
