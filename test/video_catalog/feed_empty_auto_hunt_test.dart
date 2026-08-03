import 'dart:async';

import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_hunt.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/scripted_feed_repository.dart';

void main() {
  test('an empty feed quietly keeps hunting until videos arrive', () {
    fakeAsync((async) {
      final feed = ScriptedFeedRepository(loads: [
        const [],
        const [],
        [samplePost(id: 'found')],
      ]);
      final cubit = FeedCubit(
        FeedDependencies(
          feed: feed,
          engagement: FakeVideoCatalogRepository(forYouFeed: []),
        ),
        hunt: FeedHunt(
          base: const Duration(seconds: 2),
          cap: const Duration(seconds: 30),
        ),
      );
      final states = <FeedState>[];
      final subscription = cubit.stream.listen(states.add);

      unawaited(cubit.load());
      async.flushMicrotasks();
      expect(cubit.state, isA<FeedEmpty>());

      async.elapse(const Duration(seconds: 2));
      expect(feed.loadCalls, 2);
      expect(cubit.state, isA<FeedEmpty>());

      async.elapse(const Duration(seconds: 4));
      expect(feed.loadCalls, 3);
      expect(cubit.state, isA<FeedLoaded>());

      // The hunt never flashes a loading spinner between attempts.
      expect(states.whereType<FeedLoading>(), hasLength(1));
      expect(states.whereType<FeedEmpty>(), hasLength(1));
      unawaited(subscription.cancel());
      unawaited(cubit.close());
      async.elapse(const Duration(minutes: 2));
      expect(feed.loadCalls, 3);
    });
  });
}
