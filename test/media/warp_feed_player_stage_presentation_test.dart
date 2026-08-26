import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_asset_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_representation_id.dart';

import '../../integration_test/support/warp_feed_player_stage_probe.dart';
import '../support/recording_player_preparation_feedback.dart';

void main() {
  test('presentation lookup excludes a newer attempt without a frame', () {
    var elapsed = const Duration(milliseconds: 10);
    final probe = WarpFeedPlayerStageProbe(
      RecordingPlayerPreparationFeedback(),
      () => elapsed,
    );
    final rendered = _authority('a');
    final pending = _authority('b');
    final first = probe.prepare(rendered);
    elapsed = const Duration(milliseconds: 20);
    first.firstFrameRendered();
    elapsed = const Duration(milliseconds: 25);
    probe.prepare(pending).begin();

    final evidence = probe.forPresentation(
      rendered.deliveryId,
      const Duration(milliseconds: 28),
    );

    expect(evidence?.authority, rendered);
    expect(evidence?.firstFrameAt, const Duration(milliseconds: 20));
  });
}

PlaybackAssetAuthority _authority(String identity) => PlaybackAssetAuthority(
  deliveryId: PlaybackDeliveryId.parse('delivery'),
  representationId: VideoRepresentationId.parse(identity * 64),
  assetId: PlaybackAssetId.parse(identity * 43),
);
