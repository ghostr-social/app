import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/ffi_player_preparation_feedback_port.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';

import '../support/drain_test_microtasks.dart';
import '../support/playback_authority_fixture.dart';

void main() {
  test('six proven unadmitted attempts yield to a healthy seventh', () async {
    final sent = <FfiPlayerPreparationReport>[];
    final feedback = FfiPlayerPreparationFeedbackPort(
      reportPreparation: ({required input}) async {
        sent.add(input);
        return input.postId == 'healthy'
            ? FfiPlayerPreparationDisposition.closed
            : FfiPlayerPreparationDisposition.notAdmitted;
      },
      playerCapabilityGeneration: BigInt.one,
      clientEpoch: BigInt.one,
      monotonicMicros: () => 1,
    );
    final first = feedback.prepare(testPlaybackAuthority(postId: 'bad-0'))
      ..begin();
    for (var index = 1; index < 6; index += 1) {
      feedback.prepare(testPlaybackAuthority(postId: 'bad-$index')).begin();
    }
    await drainTestMicrotasks();
    first.release();

    feedback.prepare(testPlaybackAuthority(postId: 'healthy')).begin();
    await drainTestMicrotasks();

    expect(sent.where((report) => report.postId == 'healthy'), hasLength(1));
  });

  test('released attempt stops after a proven unadmitted initial', () async {
    final first = Completer<FfiPlayerPreparationDisposition>();
    final sent = <FfiPlayerPreparationReport>[];
    final feedback = FfiPlayerPreparationFeedbackPort(
      reportPreparation: ({required input}) {
        sent.add(input);
        return sent.length == 1
            ? first.future
            : Future.value(FfiPlayerPreparationDisposition.notAdmitted);
      },
      playerCapabilityGeneration: BigInt.one,
      clientEpoch: BigInt.one,
      monotonicMicros: () => 1,
    );

    final attempt = feedback.prepare(testPlaybackAuthority())..begin();
    attempt.release();
    first.complete(FfiPlayerPreparationDisposition.notAdmitted);
    await Future<void>.delayed(const Duration(milliseconds: 40));

    expect(sent, hasLength(1));
  });
}
