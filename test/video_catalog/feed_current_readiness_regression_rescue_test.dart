import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';

import '../support/controlled_video_delivery_updates.dart';
import '../support/fake_feed_focus_port.dart';
import '../support/fakes.dart';
import '../support/feed_preparation_updates.dart';
import '../support/ready_playback_preparation.dart';
import '../support/sample_data.dart';

void main() {
  test('current player regression rearms the pending rescue', () async {
    final fixture = _Fixture();
    addTearDown(fixture.close);
    await fixture.cubit.load();
    fixture.delivery.publish(
      fixture.posts[1],
      phase: VideoDeliveryPhase.preparing,
    );
    fixture.cubit.pageChanged(1);
    await pumpEventQueue();
    expect(fixture.activeIndex, 1);
    fixture.publish(PlaybackPreparationReadiness.ready, BigInt.two);
    await pumpEventQueue();

    fixture.publish(
      PlaybackPreparationReadiness.structuralStartable,
      BigInt.from(3),
      includeNeighbor: true,
    );
    await pumpEventQueue();

    expect(fixture.activeIndex, 2);
    expect(fixture.focus.focuses.where(_isRescue), hasLength(1));
  });
}

final class _Fixture {
  final posts = List.generate(3, (index) => samplePost(id: 'p$index'));
  final delivery = ControlledVideoDeliveryUpdates();
  final preparation = ControlledPlaybackPreparationUpdates();
  final focus = FakeFeedFocusPort();
  late final repository = FakeVideoCatalogRepository(forYouFeed: posts);
  late final cubit = FeedCubit(
    FeedDependencies(
      feed: repository,
      engagement: repository,
      optional: FeedOptionalDependencies(
        focus: focus,
        delivery: FeedDeliveryDependencies(
          deliveryUpdates: delivery,
          preparationUpdates: preparation,
        ),
      ),
    ),
  );
  int get activeIndex => (cubit.state as FeedLoaded).activeIndex;

  void publish(
    PlaybackPreparationReadiness current,
    BigInt revision, {
    bool includeNeighbor = false,
  }) {
    preparation.publish(
      PlaybackPreparationPlan(
        revision: revision,
        currentDeliveryId: posts[1].media.playbackDeliveryId,
        current: _asset(posts[1].media, current),
        upcoming: [if (includeNeighbor) _asset(posts[2].media)],
      ),
    );
  }

  Future<void> close() async {
    await Future.wait([cubit.close(), delivery.close(), preparation.close()]);
  }
}

PlaybackPreparationAsset _asset(
  VideoMediaSource media, [
  PlaybackPreparationReadiness readiness = PlaybackPreparationReadiness.ready,
]) {
  final ready = readyPlaybackPreparation(media);
  return PlaybackPreparationAsset(
    authority: ready.authority,
    media: ready.media,
    readiness: readiness,
  );
}

bool _isRescue(FeedFocus focus) =>
    focus.cause == FeedFocusCause.transportRescue;
