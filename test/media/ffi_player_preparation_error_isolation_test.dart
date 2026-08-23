import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/ffi_player_preparation_feedback_port.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';

import '../support/drain_test_microtasks.dart';
import '../support/playback_authority_fixture.dart';

void main() {
  test('an ambiguous initial retries exactly before its follow-up', () async {
    final sent = <FfiPlayerPreparationReport>[];
    final feedback = FfiPlayerPreparationFeedbackPort(
      reportPreparation: ({required input}) async {
        sent.add(input);
        if (sent.length == 1) throw StateError('Rust unavailable');
        return FfiPlayerPreparationDisposition.applied;
      },
      playerCapabilityGeneration: BigInt.one,
      clientEpoch: BigInt.one,
      monotonicMicros: () => 1,
    );

    final attempt = feedback.prepare(testPlaybackAuthority())..begin();
    attempt.initialized();
    await drainTestMicrotasks();
    await Future<void>.delayed(const Duration(milliseconds: 80));

    expect(sent.map((item) => item.state), [
      FfiPlayerPreparationState.initializing,
      FfiPlayerPreparationState.initializing,
      FfiPlayerPreparationState.initialized,
    ]);
    expect(sent[1], sent.first);
  });
}
