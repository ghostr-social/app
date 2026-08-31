import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/platform/media/ffi_player_preparation_feedback_port.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';

import '../support/drain_test_microtasks.dart';
import '../support/playback_authority_fixture.dart';

void main() {
  test('progressive and HLS attempts share the eight-attempt bound', () async {
    final blocked = <Completer<FfiPlayerPreparationDisposition>>[];
    final sent = <FfiPlayerPreparationReport>[];
    final feedback = FfiPlayerPreparationFeedbackPort(
      reportPreparation: ({required input}) {
        sent.add(input);
        final completion = Completer<FfiPlayerPreparationDisposition>();
        blocked.add(completion);
        return completion.future;
      },
      playerCapabilityGeneration: BigInt.one,
      clientEpoch: BigInt.one,
      monotonicMicros: () => 1,
    );

    for (var index = 0; index < 4; index += 1) {
      feedback.prepare(testPlaybackAuthority(postId: 'p-$index')).begin();
    }
    for (var index = 0; index < 5; index += 1) {
      feedback.prepareHls(_hlsAuthority(index)).begin();
    }

    expect(blocked, hasLength(8));
    expect(
      sent.where((report) => report.assetId.startsWith('hls-v1:')),
      hasLength(4),
    );
    for (final completion in blocked) {
      completion.complete(FfiPlayerPreparationDisposition.applied);
    }
    await drainTestMicrotasks(12);
  });
}

HlsPlaybackAuthority _hlsAuthority(int index) => HlsPlaybackAuthority(
  deliveryId: PlaybackDeliveryId.parse('h-$index'),
  representationId: VideoRepresentationId.parse('a' * 64),
  assetRevision: HlsPlaybackAssetRevision.parse(BigInt.from(index + 1)),
);
