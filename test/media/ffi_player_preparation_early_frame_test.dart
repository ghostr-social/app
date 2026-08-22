import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/ffi_player_preparation_feedback_port.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';

import '../support/drain_test_microtasks.dart';
import '../support/playback_authority_fixture.dart';

void main() {
  test('an early native frame waits for plugin settlement', () async {
    final sent = <FfiPlayerPreparationReport>[];
    final feedback = FfiPlayerPreparationFeedbackPort(
      reportPreparation: ({required input}) async => sent.add(input),
      playerCapabilityGeneration: BigInt.one,
      clientEpoch: BigInt.one,
      monotonicMicros: () => 1,
    );
    final attempt = feedback.prepare(testPlaybackAuthority())..begin();
    await drainTestMicrotasks();

    attempt.firstFrameRendered();
    attempt.initialized();
    await drainTestMicrotasks();

    expect(sent.map((item) => item.state), [
      FfiPlayerPreparationState.initializing,
      FfiPlayerPreparationState.initialized,
      FfiPlayerPreparationState.firstFrameRendered,
    ]);
  });
}
