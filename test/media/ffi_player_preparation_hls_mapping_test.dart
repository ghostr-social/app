import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/platform/media/ffi_player_preparation_feedback_port.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';

import '../support/drain_test_microtasks.dart';

void main() {
  test(
    'maps exact HLS authority through the bounded preparation channel',
    () async {
      final sent = <FfiPlayerPreparationReport>[];
      final feedback = FfiPlayerPreparationFeedbackPort(
        reportPreparation: ({required input}) async {
          sent.add(input);
          return FfiPlayerPreparationDisposition.applied;
        },
        playerCapabilityGeneration: BigInt.one,
        clientEpoch: BigInt.one,
        monotonicMicros: () => 1,
      );
      final attempt = feedback.prepareHls(_authority(BigInt.from(7)))..begin();
      await drainTestMicrotasks();
      attempt.initialized();
      attempt.firstFrameRendered();
      attempt.release();
      await drainTestMicrotasks();

      expect(sent.map((item) => item.state), [
        FfiPlayerPreparationState.initializing,
        FfiPlayerPreparationState.initialized,
        FfiPlayerPreparationState.firstFrameRendered,
        FfiPlayerPreparationState.released,
      ]);
      expect(sent.last.postId, 'post-hls');
      expect(sent.last.representationId, 'a' * 64);
      expect(sent.last.assetId, 'hls-v1:7');
    },
  );

  test('rejects an HLS revision outside the native u64 authority', () {
    final feedback = FfiPlayerPreparationFeedbackPort(
      reportPreparation: ({required input}) async =>
          FfiPlayerPreparationDisposition.applied,
    );

    expect(
      () => feedback.prepareHls(_authority(BigInt.one << 64)),
      throwsArgumentError,
    );
  });
}

HlsPlaybackAuthority _authority(BigInt revision) => HlsPlaybackAuthority(
  deliveryId: PlaybackDeliveryId.parse('post-hls'),
  representationId: VideoRepresentationId.parse('a' * 64),
  assetRevision: HlsPlaybackAssetRevision.parse(revision),
);
