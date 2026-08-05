import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fake_feed_focus_port.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('loading a feed focuses the engine on the first post', () async {
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

    final focus = focusPort.focuses.single;
    expect(focus.currentIndex, 0);
    expect(focus.watched, Duration.zero);
    expect(
      focus.window.map((post) => post.media.remoteUrl),
      [
        for (var index = 0; index <= 6; index += 1)
          'https://example.com/video/post-$index.mp4',
      ],
    );
  });
}
