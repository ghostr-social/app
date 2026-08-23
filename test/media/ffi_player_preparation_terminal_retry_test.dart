import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/ffi_player_preparation_feedback_port.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';

import '../support/playback_authority_fixture.dart';

void main() {
  test('an ambiguous terminal cannot block a later attempt', () async {
    final sent = <FfiPlayerPreparationReport>[];
    var terminalFailed = false;
    final feedback = FfiPlayerPreparationFeedbackPort(
      reportPreparation: ({required input}) async {
        sent.add(input);
        if (input.state == FfiPlayerPreparationState.released &&
            !terminalFailed) {
          terminalFailed = true;
          return FfiPlayerPreparationDisposition.unavailable;
        }
        return FfiPlayerPreparationDisposition.applied;
      },
      playerCapabilityGeneration: BigInt.one,
      clientEpoch: BigInt.one,
      monotonicMicros: () => 1,
    );

    final first = feedback.prepare(testPlaybackAuthority(postId: 'first'));
    first.begin();
    first.release();
    feedback.prepare(testPlaybackAuthority(postId: 'second')).begin();
    await Future<void>.delayed(const Duration(milliseconds: 80));

    expect(sent.map((item) => '${item.postId}:${item.state.name}'), [
      'first:initializing',
      'second:initializing',
      'first:released',
      'first:released',
    ]);
    expect(sent[3], sent[2]);
  });
}
