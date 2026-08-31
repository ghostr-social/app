import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_representation_id.dart';

import '../../integration_test/support/warp_feed_player_stage_probe.dart';
import '../support/recording_player_preparation_feedback.dart';

void main() {
  test('player stage probe records exact successful HLS lifecycle', () {
    var elapsed = Duration.zero;
    final delegate = RecordingPlayerPreparationFeedback();
    final probe = WarpFeedPlayerStageProbe(delegate, () => elapsed);
    final authority = _authority();
    final attempt = probe.prepareHls(authority);

    elapsed = const Duration(milliseconds: 10);
    attempt.begin();
    elapsed = const Duration(milliseconds: 20);
    attempt.initialized();
    elapsed = const Duration(milliseconds: 30);
    attempt.firstFrameRendered();
    elapsed = const Duration(milliseconds: 40);
    attempt.release();

    final evidence = probe.hlsAttemptsFor(authority).single;
    expect(evidence.authority, same(authority));
    expect(evidence.lifecycle, const [
      WarpFeedPlayerStage.initializing,
      WarpFeedPlayerStage.initialized,
      WarpFeedPlayerStage.firstFrameRendered,
      WarpFeedPlayerStage.released,
    ]);
    expect(evidence.initializingAt, const Duration(milliseconds: 10));
    expect(evidence.initializedAt, const Duration(milliseconds: 20));
    expect(evidence.firstFrameAt, const Duration(milliseconds: 30));
    expect(evidence.failedAt, isNull);
    expect(evidence.releasedAt, const Duration(milliseconds: 40));
    expect(delegate.hlsStatesFor(authority), const [
      RecordedPreparationState.initializing,
      RecordedPreparationState.initialized,
      RecordedPreparationState.firstFrameRendered,
      RecordedPreparationState.released,
    ]);
  });
}

HlsPlaybackAuthority _authority() => HlsPlaybackAuthority(
  deliveryId: PlaybackDeliveryId.parse('hls-delivery'),
  representationId: VideoRepresentationId.parse('a' * 64),
  assetRevision: HlsPlaybackAssetRevision.parse(BigInt.one),
);
