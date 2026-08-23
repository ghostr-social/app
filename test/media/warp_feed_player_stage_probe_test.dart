import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_asset_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_representation_id.dart';

import '../../integration_test/support/warp_feed_player_stage_probe.dart';
import '../support/recording_player_preparation_feedback.dart';

void main() {
  test(
    'player stage probe preserves feedback and records monotonic stages',
    () {
      var elapsed = Duration.zero;
      final delegate = RecordingPlayerPreparationFeedback();
      final probe = WarpFeedPlayerStageProbe(delegate, () => elapsed);
      final authority = _authority('delivery');

      final attempt = probe.prepare(authority);
      elapsed = const Duration(milliseconds: 10);
      attempt.begin();
      elapsed = const Duration(milliseconds: 30);
      attempt.initialized();
      elapsed = const Duration(milliseconds: 40);
      attempt.firstFrameRendered();
      elapsed = const Duration(milliseconds: 50);
      attempt.release();

      final evidence = probe.latestFor(authority.deliveryId)!;
      expect(evidence.preparedAt, Duration.zero);
      expect(evidence.initializingAt, const Duration(milliseconds: 10));
      expect(evidence.initializedAt, const Duration(milliseconds: 30));
      expect(evidence.firstFrameAt, const Duration(milliseconds: 40));
      expect(evidence.releasedAt, const Duration(milliseconds: 50));
      expect(delegate.events.map((event) => event.state), [
        RecordedPreparationState.initializing,
        RecordedPreparationState.initialized,
        RecordedPreparationState.firstFrameRendered,
        RecordedPreparationState.released,
      ]);
    },
  );

  test('player stage lookup excludes attempts after a presentation', () {
    var elapsed = const Duration(milliseconds: 10);
    final probe = WarpFeedPlayerStageProbe(
      RecordingPlayerPreparationFeedback(),
      () => elapsed,
    );
    final authority = _authority('delivery');
    final first = probe.prepare(authority);
    first.firstFrameRendered();
    elapsed = const Duration(milliseconds: 30);
    probe.prepare(authority).firstFrameRendered();

    expect(
      probe
          .latestFor(
            authority.deliveryId,
            noLaterThan: const Duration(milliseconds: 20),
          )
          ?.firstFrameAt,
      const Duration(milliseconds: 10),
    );
    first.release();
  });
}

PlaybackAssetAuthority _authority(String deliveryId) => PlaybackAssetAuthority(
  deliveryId: PlaybackDeliveryId.parse(deliveryId),
  representationId: VideoRepresentationId.parse(
    'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
  ),
  assetId: PlaybackAssetId.parse('aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'),
);
