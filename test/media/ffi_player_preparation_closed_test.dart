import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/ffi_player_preparation_feedback_port.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';

import '../support/drain_test_microtasks.dart';
import '../support/playback_authority_fixture.dart';

void main() {
  test('a proven closed manager permanently silences feedback', () async {
    final sent = <FfiPlayerPreparationReport>[];
    final feedback = FfiPlayerPreparationFeedbackPort(
      reportPreparation: ({required input}) async {
        sent.add(input);
        return sent.length == 1
            ? FfiPlayerPreparationDisposition.unavailable
            : FfiPlayerPreparationDisposition.closed;
      },
      playerCapabilityGeneration: BigInt.one,
      clientEpoch: BigInt.one,
      monotonicMicros: () => 1,
    );

    final active = feedback.prepare(testPlaybackAuthority())..begin();
    active.initialized();
    await Future<void>.delayed(const Duration(milliseconds: 80));
    active.release();
    feedback.prepare(testPlaybackAuthority(postId: 'later')).begin();
    await drainTestMicrotasks();

    expect(sent, hasLength(2));
    expect(sent[1], sent.first);
  });
}
