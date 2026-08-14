import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fake_feed_focus_port.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'refresh republishes the complete changed roster at the same index',
    () async {
      final repository = FakeVideoCatalogRepository(
        forYouFeed: [
          samplePost(id: 'current'),
          samplePost(id: 'next'),
        ],
      );
      final focus = FakeFeedFocusPort();
      final cubit = FeedCubit(
        FeedDependencies(
          feed: repository,
          engagement: repository,
          optional: FeedOptionalDependencies(focus: focus),
        ),
      );
      addTearDown(cubit.close);
      await cubit.load();
      repository.forYouFeed.insert(0, samplePost(id: 'fresh'));

      await cubit.refresh();

      expect(focus.focuses, hasLength(2));
      expect(focus.focuses.last.current.id.value, 'current');
      expect(focus.focuses.last.window.map((post) => post.id.value), [
        'current',
        'next',
        'fresh',
      ]);
      expect(focus.focuses.last.watched, Duration.zero);
    },
  );
}
