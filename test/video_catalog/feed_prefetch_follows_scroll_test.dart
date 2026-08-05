import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fake_feed_focus_port.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('scrolling refocuses the delivery window around the viewer', () async {
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

    cubit.pageChanged(6);
    await pumpEventQueue();

    final focus = focusPort.focuses.last;
    expect(
      focus.window.map((post) => post.media.remoteUrl),
      [
        for (var index = 4; index <= 11; index += 1)
          'https://example.com/video/post-$index.mp4',
      ],
    );
    expect(focus.currentIndex, 2);
    expect(
      focus.current.media.remoteUrl,
      'https://example.com/video/post-6.mp4',
    );
  });
}
