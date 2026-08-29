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
  test('rescue skips structural bytes for a player-verified frame', () async {
    final fixture = _Fixture();
    addTearDown(fixture.close);
    await fixture.cubit.load();
    expect((fixture.cubit.state as FeedLoaded).activeIndex, 0);
    fixture.publishEvidence();

    fixture.cubit.pageChanged(1);
    await pumpEventQueue();

    final loaded = fixture.cubit.state as FeedLoaded;
    expect(loaded.activeIndex, 3);
    expect(loaded.roster.active.id.value, 'p3');
    final rescue = fixture.focus.focuses.where(_isRescue).single;
    expect(rescue.current.id.value, 'p3');
    expect(rescue.rescue?.reason, FeedTransportRescueReason.etaUnavailable);
    expect(rescue.rescue?.rankDisplacement, 2);
    expect(rescue.rescue?.wait, Duration.zero);
  });
}

final class _Fixture {
  final posts = List.generate(4, (index) => samplePost(id: 'p$index'));
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

  void publishEvidence() {
    delivery.publish(posts[1], phase: VideoDeliveryPhase.startable);
    delivery.publish(posts[2], phase: VideoDeliveryPhase.startable);
    preparation.publish(
      PlaybackPreparationPlan(
        revision: BigInt.one,
        currentDeliveryId: posts[0].media.playbackDeliveryId,
        upcoming: [
          _structural(posts[1].media),
          _structural(posts[2].media),
          readyPlaybackPreparation(posts[3].media),
        ],
      ),
    );
  }

  Future<void> close() async {
    await Future.wait([cubit.close(), delivery.close(), preparation.close()]);
  }
}

PlaybackPreparationAsset _structural(VideoMediaSource media) {
  final ready = readyPlaybackPreparation(media);
  return PlaybackPreparationAsset(
    authority: ready.authority,
    sourceRepresentationId: ready.sourceRepresentationId,
    media: ready.media,
    readiness: PlaybackPreparationReadiness.structuralStartable,
  );
}

bool _isRescue(FeedFocus focus) =>
    focus.cause == FeedFocusCause.transportRescue;
