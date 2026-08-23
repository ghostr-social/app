import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/ffi_player_preparation_feedback_port.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';

import '../support/drain_test_microtasks.dart';
import '../support/playback_authority_fixture.dart';

void main() {
  test('a permanent rejection silences the rejected attempt', () async {
    final sent = <FfiPlayerPreparationReport>[];
    final feedback = FfiPlayerPreparationFeedbackPort(
      reportPreparation: ({required input}) async {
        sent.add(input);
        return FfiPlayerPreparationDisposition.rejected;
      },
      playerCapabilityGeneration: BigInt.one,
      clientEpoch: BigInt.one,
      monotonicMicros: () => 1,
    );

    final attempt = feedback.prepare(testPlaybackAuthority())..begin();
    await drainTestMicrotasks();
    attempt.initialized();
    attempt.release();
    await drainTestMicrotasks();

    expect(sent, hasLength(1));
    expect(sent.single.state, FfiPlayerPreparationState.initializing);
  });
}
