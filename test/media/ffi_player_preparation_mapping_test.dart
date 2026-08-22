import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/ffi_player_preparation_feedback_port.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';

import '../support/drain_test_microtasks.dart';
import '../support/playback_authority_fixture.dart';

void main() {
  test('maps one authority-fenced attempt with ordered observations', () async {
    final sent = <FfiPlayerPreparationReport>[];
    var time = 40;
    final feedback = FfiPlayerPreparationFeedbackPort(
      reportPreparation: ({required input}) async => sent.add(input),
      playerCapabilityGeneration: BigInt.from(7),
      clientEpoch: BigInt.from(11),
      monotonicMicros: () => ++time,
    );
    final attempt = feedback.prepare(testPlaybackAuthority())..begin();
    await drainTestMicrotasks();
    attempt.initialized();
    await drainTestMicrotasks();
    attempt.firstFrameRendered();
    await drainTestMicrotasks();
    attempt.release();
    await drainTestMicrotasks();

    expect(sent.map((item) => item.state), [
      FfiPlayerPreparationState.initializing,
      FfiPlayerPreparationState.initialized,
      FfiPlayerPreparationState.firstFrameRendered,
      FfiPlayerPreparationState.released,
    ]);
    expect(sent.map((item) => item.sequence), [
      BigInt.one,
      BigInt.two,
      BigInt.from(3),
      BigInt.from(4),
    ]);
    expect(sent.map((item) => item.observedMonotonicUs), [
      BigInt.from(41),
      BigInt.from(42),
      BigInt.from(43),
      BigInt.from(44),
    ]);
    expect(sent.last.postId, 'post-1');
    expect(
      sent.last.representationId,
      testPlaybackAuthority().representationId.value,
    );
    expect(sent.last.assetId, testPlaybackCapability);
    expect(sent.last.playerCapabilityGeneration, BigInt.from(7));
    expect(sent.last.clientEpoch, BigInt.from(11));
  });
}
