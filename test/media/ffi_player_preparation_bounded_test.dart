import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/ffi_player_preparation_feedback_port.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';

import '../support/drain_test_microtasks.dart';
import '../support/playback_authority_fixture.dart';

void main() {
  test('rapid unique attempts retain only a bounded latest tail', () async {
    final first = Completer<void>();
    final sent = <FfiPlayerPreparationReport>[];
    final feedback = FfiPlayerPreparationFeedbackPort(
      reportPreparation: ({required input}) {
        sent.add(input);
        return sent.length == 1 ? first.future : Future.value();
      },
      playerCapabilityGeneration: BigInt.one,
      clientEpoch: BigInt.one,
      monotonicMicros: () => 1,
    );

    for (var index = 0; index < 8; index += 1) {
      feedback.prepare(testPlaybackAuthority(postId: 'post-$index')).begin();
    }
    expect(sent, hasLength(1));
    first.complete();
    await drainTestMicrotasks(12);

    expect(sent.length, lessThanOrEqualTo(4));
    expect(sent.last.postId, 'post-7');
  });
}
