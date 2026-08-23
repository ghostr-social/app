import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/ffi_player_preparation_feedback_port.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';

import '../support/drain_test_microtasks.dart';
import '../support/playback_authority_fixture.dart';

void main() {
  test('a seventh concurrent attempt stays silent after rejection', () async {
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
    for (var index = 0; index < 6; index += 1) {
      feedback.prepare(testPlaybackAuthority(postId: 'active-$index')).begin();
    }
    final dropped = feedback.prepare(testPlaybackAuthority(postId: 'dropped'));
    dropped.begin();
    dropped.release();
    await drainTestMicrotasks();

    expect(sent, hasLength(6));
    expect(sent.where((report) => report.postId == 'dropped'), isEmpty);
    for (final completion in blocked) {
      completion.complete(FfiPlayerPreparationDisposition.applied);
    }
    await drainTestMicrotasks();
  });
}
