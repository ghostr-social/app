import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_asset_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_representation_id.dart';

import '../../integration_test/support/warp_feed_player_stage_probe.dart';
import '../../integration_test/support/warp_origin_timeout_fallback_scenario.dart';
import '../support/recording_player_preparation_feedback.dart';

void main() {
  test('fallback evidence selects the live decoded retry', () {
    final probe = WarpFeedPlayerStageProbe(
      RecordingPlayerPreparationFeedback(),
      () => Duration.zero,
    );
    final authority = _authority();
    final retired = probe.prepare(authority)..firstFrameRendered();
    retired.release();
    final live = probe.prepare(authority)..firstFrameRendered();

    final selected = warpOriginTimeoutDecodedStage(
      probe.attemptsFor(authority.deliveryId),
      authority,
    );

    expect(selected, same(probe.attemptsFor(authority.deliveryId).last));
    expect(selected?.isTerminal, isFalse);
    live.release();
  });
}

PlaybackAssetAuthority _authority() => PlaybackAssetAuthority(
  deliveryId: PlaybackDeliveryId.parse('next'),
  representationId: VideoRepresentationId.parse('a' * 64),
  assetId: PlaybackAssetId.parse('a' * 43),
);
