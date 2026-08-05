import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/controllable_video_feed_updates.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/scripted_feed_repository.dart';

void main() {
  test(
    'refreshing Following rebinds its creator-scoped update stream',
    () async {
      final updates = ControllableVideoFeedUpdates();
      addTearDown(updates.close);
      final cubit = FeedCubit(
        FeedDependencies(
          feed: ScriptedFeedRepository(
            loads: [
              [samplePost(id: 'first')],
              [samplePost(id: 'second')],
            ],
          ),
          engagement: FakeVideoCatalogRepository(forYouFeed: []),
          optional: FeedOptionalDependencies(updates: updates),
        ),
      );
      addTearDown(cubit.close);

      await cubit.load(FeedKind.following);
      await cubit.refresh();

      expect(updates.watchedKinds, [FeedKind.following, FeedKind.following]);
    },
  );
}
