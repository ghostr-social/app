import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fake_social_graph_repository.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('an accepted hidden block still removes the creator', () async {
    final blocked = samplePost(
      id: 'blocked',
      creator: sampleCreator(id: 'blocked-creator'),
    );
    final replacement = samplePost(id: 'replacement');
    final source = FakeVideoCatalogRepository(
      forYouFeed: [blocked, replacement],
    );
    final social = _GatedSocialGraph();
    final history = FakeWatchHistoryRepository();
    final cubit = FeedCubit(
      FeedDependencies(
        feed: source,
        engagement: source,
        optional: FeedOptionalDependencies(
          social: social,
          watch: FeedWatchDependencies(
            tracker: WatchHistoryTracker(
              history: history,
              failureReporter: RecordingFailureReporter(),
            ),
          ),
        ),
      ),
    );
    addTearDown(() async {
      if (!social.release.isCompleted) social.release.complete();
      await cubit.close();
    });
    await cubit.load();

    final blocking = cubit.blockCreator(blocked);
    await social.started.future;
    cubit.surfaceVisibilityChanged(false);
    social.release.complete();
    await blocking;

    final hidden = cubit.state as FeedLoaded;
    expect(hidden.posts.single.id.value, 'replacement');
    cubit.surfaceVisibilityChanged(true);
    await pumpEventQueue();
    expect((cubit.state as FeedLoaded).posts.single.id.value, 'replacement');
    expect(history.entries.map((entry) => entry.videoId), [
      'e:replacement',
      'e:blocked',
    ]);
  });
}

final class _GatedSocialGraph extends FakeSocialGraphRepository {
  final started = Completer<void>();
  final release = Completer<void>();

  @override
  Future<bool> toggleBlock(ProfileId profileId) async {
    started.complete();
    await release.future;
    return super.toggleBlock(profileId);
  }
}
