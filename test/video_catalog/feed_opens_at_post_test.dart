import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fake_feed_focus_port.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a feed opens on the requested video and reports its focus', () async {
    final posts = [samplePost(id: 'clip-1'), samplePost(id: 'clip-2')];
    final repository = FakeVideoCatalogRepository(forYouFeed: posts);
    final focus = FakeFeedFocusPort();
    final cubit = FeedCubit(
      FeedDependencies(
        feed: repository,
        engagement: repository,
        optional: FeedOptionalDependencies(focus: focus),
      ),
      openAt: posts[1].id,
    );
    addTearDown(cubit.close);

    await cubit.load();

    expect((cubit.state as FeedLoaded).activeIndex, 1);
    expect(focus.focuses.last.currentIndex, 1);
  });
}
