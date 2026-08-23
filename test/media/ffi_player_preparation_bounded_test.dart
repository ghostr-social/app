import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/ffi_player_preparation_feedback_port.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';

import '../support/drain_test_microtasks.dart';
import '../support/playback_authority_fixture.dart';

void main() {
  test('all six controller attempts survive a blocked reporter', () async {
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

    for (var index = 0; index < 6; index += 1) {
      feedback.prepare(testPlaybackAuthority(postId: 'post-$index')).begin();
    }
    expect(sent, hasLength(1));
    first.complete();
    await drainTestMicrotasks(12);

    expect(sent, hasLength(6));
    expect(sent.last.postId, 'post-5');
  });

  test('a dispatched initial keeps its queued terminal under churn', () async {
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
    final active = feedback.prepare(testPlaybackAuthority(postId: 'active'));
    active.begin();
    active.release();
    for (var index = 0; index < 6; index += 1) {
      feedback.prepare(testPlaybackAuthority(postId: 'churn-$index')).begin();
    }

    first.complete();
    await drainTestMicrotasks(16);

    expect(
      sent
          .where((report) => report.postId == 'active')
          .map((report) => report.state),
      [
        FfiPlayerPreparationState.initializing,
        FfiPlayerPreparationState.released,
      ],
    );
  });
}
