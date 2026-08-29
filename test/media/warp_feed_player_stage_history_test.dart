import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_asset_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_representation_id.dart';

import '../../integration_test/support/warp_feed_player_stage_probe.dart';
import '../support/recording_player_preparation_feedback.dart';

void main() {
  test('player stage history retains every attempt for one delivery', () {
    var elapsed = Duration.zero;
    final probe = WarpFeedPlayerStageProbe(
      RecordingPlayerPreparationFeedback(),
      () => elapsed,
    );
    final authority = _authority();
    final first = probe.prepare(authority)..begin();
    elapsed = const Duration(milliseconds: 10);
    first.firstFrameRendered();
    elapsed = const Duration(milliseconds: 20);
    first.release();
    probe.prepare(authority).begin();

    final attempts = probe.attemptsFor(authority.deliveryId);

    expect(attempts, hasLength(2));
    expect(attempts.first.firstFrameAt, const Duration(milliseconds: 10));
    expect(attempts.first.releasedAt, const Duration(milliseconds: 20));
    expect(attempts.last.initializingAt, const Duration(milliseconds: 20));
  });
}

PlaybackAssetAuthority _authority() => PlaybackAssetAuthority(
  deliveryId: PlaybackDeliveryId.parse('delivery'),
  representationId: VideoRepresentationId.parse('a' * 64),
  assetId: PlaybackAssetId.parse('a' * 43),
);
