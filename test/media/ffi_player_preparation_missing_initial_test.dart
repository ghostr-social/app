import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/ffi_player_preparation_feedback_port.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';

import '../support/drain_test_microtasks.dart';
import '../support/playback_authority_fixture.dart';

void main() {
  test('missing initial replays only exact acknowledged history', () async {
    final sent = <FfiPlayerPreparationReport>[];
    var missingReported = false;
    final feedback = FfiPlayerPreparationFeedbackPort(
      reportPreparation: ({required input}) async {
        sent.add(input);
        if (input.state == FfiPlayerPreparationState.firstFrameRendered &&
            !missingReported) {
          missingReported = true;
          return FfiPlayerPreparationDisposition.missingInitial;
        }
        return FfiPlayerPreparationDisposition.applied;
      },
      playerCapabilityGeneration: BigInt.one,
      clientEpoch: BigInt.one,
      monotonicMicros: () => sent.length + 1,
    );

    final attempt = feedback.prepare(testPlaybackAuthority())..begin();
    await drainTestMicrotasks();
    attempt.initialized();
    await drainTestMicrotasks();
    attempt.firstFrameRendered();
    await drainTestMicrotasks(12);

    expect(sent.map((report) => report.sequence), [
      BigInt.one,
      BigInt.two,
      BigInt.from(3),
      BigInt.one,
      BigInt.two,
      BigInt.from(3),
    ]);
    expect(sent[3], sent[0]);
    expect(sent[4], sent[1]);
    expect(sent[5], sent[2]);
  });
}
