import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fake_feed_focus_port.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('scrolling refocuses the delivery window around the viewer', () async {
    final focusPort = FakeFeedFocusPort();
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [
        for (var index = 0; index < 50; index += 1)
          samplePost(id: 'post-$index'),
      ],
    );
    final cubit = FeedCubit(
      FeedDependencies(
        feed: repository,
        engagement: repository,
        optional: FeedOptionalDependencies(focus: focusPort),
      ),
    );
    addTearDown(cubit.close);
    await cubit.load();

    cubit.pageChanged(25);
    await pumpEventQueue();

    final focus = focusPort.focuses.last;
    expect((cubit.state as FeedLoaded).posts, hasLength(50));
    expect(focus.window.map((post) => post.media.remoteUrl), [
      for (var index = 22; index < 50; index += 1)
        'https://example.com/video/post-$index.mp4',
    ]);
    expect(focus.currentIndex, 3);
    expect(
      focus.current.media.remoteUrl,
      'https://example.com/video/post-25.mp4',
    );
  });
}
