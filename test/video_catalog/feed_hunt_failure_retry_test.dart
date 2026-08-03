import 'dart:async';

import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_hunt.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/scripted_feed_repository.dart';

void main() {
  test('a failing hunt attempt never surfaces and the hunt goes on', () {
    fakeAsync((async) {
      final feed = ScriptedFeedRepository(loads: [
        const [],
        null,
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

      unawaited(cubit.load());
      async.flushMicrotasks();
      expect(cubit.state, isA<FeedEmpty>());

      async.elapse(const Duration(seconds: 2));
      expect(feed.loadCalls, 2);
      expect(cubit.state, isA<FeedEmpty>());

      async.elapse(const Duration(seconds: 4));
      expect(feed.loadCalls, 3);
      expect(cubit.state, isA<FeedLoaded>());
      unawaited(cubit.close());
    });
  });
}
