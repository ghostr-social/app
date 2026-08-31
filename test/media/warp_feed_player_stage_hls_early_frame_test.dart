import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_representation_id.dart';

import '../../integration_test/support/warp_feed_player_stage_probe.dart';
import '../support/recording_player_preparation_feedback.dart';

void main() {
  test('HLS probe publishes an early frame after plugin readiness', () {
    var elapsed = Duration.zero;
    final authority = _authority();
    final probe = WarpFeedPlayerStageProbe(
      RecordingPlayerPreparationFeedback(),
      () => elapsed,
    );
    final attempt = probe.prepareHls(authority);

    attempt.begin();
    elapsed = const Duration(milliseconds: 10);
    attempt.firstFrameRendered();
    elapsed = const Duration(milliseconds: 20);
    attempt.initialized();
    elapsed = const Duration(milliseconds: 30);
    attempt.release();

    final evidence = probe.hlsAttemptsFor(authority).single;
    expect(evidence.firstFrameAt, const Duration(milliseconds: 10));
    expect(evidence.lifecycle, const [
      WarpFeedPlayerStage.initializing,
      WarpFeedPlayerStage.initialized,
      WarpFeedPlayerStage.firstFrameRendered,
      WarpFeedPlayerStage.released,
    ]);
  });
}

HlsPlaybackAuthority _authority() => HlsPlaybackAuthority(
  deliveryId: PlaybackDeliveryId.parse('hls-delivery'),
  representationId: VideoRepresentationId.parse('a' * 64),
  assetRevision: HlsPlaybackAssetRevision.parse(BigInt.one),
);
