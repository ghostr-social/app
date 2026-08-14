import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fake_feed_focus_port.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('blocking a creator removes their videos and keeps the rest', () async {
    final blocked = sampleCreator(id: 'creator-blocked', displayName: 'Spam');
    final kept = sampleCreator(id: 'creator-kept', displayName: 'Nora');
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [
        samplePost(id: 'post-1', creator: blocked),
        samplePost(id: 'post-2', creator: kept),
        samplePost(id: 'post-3', creator: blocked),
      ],
    );
    final focus = FakeFeedFocusPort();
    final cubit = FeedCubit(
      FeedDependencies(
        feed: repository,
        engagement: repository,
        optional: FeedOptionalDependencies(social: repository, focus: focus),
      ),
    );
    addTearDown(cubit.close);
    await cubit.load();

    await cubit.blockCreator(repository.forYouFeed.first);

    final state = cubit.state as FeedLoaded;
    expect(state.posts, hasLength(1));
    expect(state.posts.single.creator.id, kept.id);
    expect(state.notice, 'Blocked ${blocked.handle}');
    expect(repository.blockedProfiles, contains(blocked.id));
    expect(focus.focuses, hasLength(2));
    expect(focus.focuses.last.window, state.posts);
    expect(focus.focuses.last.currentIndex, 0);
  });
}
