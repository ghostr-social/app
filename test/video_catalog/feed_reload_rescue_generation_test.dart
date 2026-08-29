import 'dart:async';

import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/controlled_video_delivery_updates.dart';
import '../support/fakes.dart';
import '../support/feed_preparation_updates.dart';
import '../support/player_verified_preparation.dart';
import '../support/sample_data.dart';

void main() {
  test('reload fences the previous roster rescue grace', () {
    fakeAsync((clock) {
      final fixture = _Fixture();
      fixture.load(clock);
      fixture.armRescue(clock);
      fixture.reloadWithSameDelivery(clock);

      clock.elapse(const Duration(milliseconds: 250));

      expect(fixture.activeId, 'p1');
      fixture.close(clock);
    });
  });
}

final class _Fixture {
  final posts = List.generate(3, (index) => samplePost(id: 'p$index'));
  final delivery = ControlledVideoDeliveryUpdates();
  final preparation = ControlledPlaybackPreparationUpdates();
  late final repository = FakeVideoCatalogRepository(
    forYouFeed: List<VideoPost>.of(posts),
  );
  late final cubit = FeedCubit(
    FeedDependencies(
      feed: repository,
      engagement: repository,
      optional: FeedOptionalDependencies(
        delivery: FeedDeliveryDependencies(
          deliveryUpdates: delivery,
          preparationUpdates: preparation,
        ),
      ),
    ),
  );

  String get activeId => (cubit.state as FeedLoaded).roster.active.id.value;

  void load(FakeAsync clock) {
    unawaited(cubit.load());
    clock.flushMicrotasks();
  }

  void armRescue(FakeAsync clock) {
    delivery.publish(
      posts[1],
      phase: VideoDeliveryPhase.preparing,
      eta: const Duration(milliseconds: 100),
    );
    preparation.publish(
      playerVerifiedPlan(posts, currentIndex: 0, readyIndices: [2]),
    );
    cubit.pageChanged(1);
    clock.flushMicrotasks();
    expect(activeId, 'p1');
  }

  void reloadWithSameDelivery(FakeAsync clock) {
    repository.forYouFeed
      ..clear()
      ..addAll([posts[1], posts[2], posts[0]]);
    unawaited(cubit.reload());
    clock.flushMicrotasks();
    expect(activeId, 'p1');
  }

  void close(FakeAsync clock) {
    unawaited(
      Future.wait([cubit.close(), delivery.close(), preparation.close()]),
    );
    clock.flushMicrotasks();
  }
}
