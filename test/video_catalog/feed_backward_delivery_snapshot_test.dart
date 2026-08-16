import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/controllable_video_delivery_updates.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('backward navigation never substitutes another ready video', () async {
    final posts = List.generate(26, (index) => samplePost(id: 'p$index'));
    final updates = ControllableVideoDeliveryUpdates();
    final repository = FakeVideoCatalogRepository(forYouFeed: posts);
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
    updates.publish(posts[20], startable: true);
    updates.publish(posts[21], startable: false, etaMilliseconds: 1000);
    for (var index = 1; index <= 25; index += 1) {
      cubit.pageChanged(index);
    }

    for (var index = 24; index >= 21; index -= 1) {
      cubit.pageChanged(index);
    }

    expect((cubit.state as FeedLoaded).activeIndex, 21);
  });
}
