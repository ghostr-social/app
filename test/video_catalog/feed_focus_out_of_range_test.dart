import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fake_feed_focus_port.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('an out-of-range page keeps the delivery focus unchanged', () async {
    final focusPort = FakeFeedFocusPort();
    final repository = FakeVideoCatalogRepository(forYouFeed: [
      for (var index = 0; index < 12; index += 1) samplePost(id: 'post-$index'),
    ]);
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
      optional: FeedOptionalDependencies(focus: focusPort),
    ));
    addTearDown(cubit.close);
    await cubit.load();

    cubit.pageChanged(12);
    cubit.pageChanged(-1);
    await pumpEventQueue();

    expect(focusPort.focuses, hasLength(1));
    expect(focusPort.focuses.single.currentIndex, 0);
  });
}
