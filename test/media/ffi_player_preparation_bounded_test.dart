import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/ffi_player_preparation_feedback_port.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';

import '../support/drain_test_microtasks.dart';
import '../support/playback_authority_fixture.dart';

void main() {
  test('all six controller attempts survive a blocked reporter', () async {
    final first = Completer<FfiPlayerPreparationDisposition>();
    final sent = <FfiPlayerPreparationReport>[];
    final feedback = FfiPlayerPreparationFeedbackPort(
      reportPreparation: ({required input}) {
        sent.add(input);
        return sent.length == 1
            ? first.future
            : Future.value(FfiPlayerPreparationDisposition.applied);
      },
      playerCapabilityGeneration: BigInt.one,
      clientEpoch: BigInt.one,
      monotonicMicros: () => 1,
    );

    for (var index = 0; index < 6; index += 1) {
      feedback.prepare(testPlaybackAuthority(postId: 'post-$index')).begin();
    }
    await drainTestMicrotasks();
    expect(sent, hasLength(6));
    first.complete(FfiPlayerPreparationDisposition.applied);
    await drainTestMicrotasks(12);

    expect(sent, hasLength(6));
    expect(sent.last.postId, 'post-5');
  });

  test('a dispatched initial keeps its queued terminal under churn', () async {
    final first = Completer<FfiPlayerPreparationDisposition>();
    final sent = <FfiPlayerPreparationReport>[];
    final feedback = FfiPlayerPreparationFeedbackPort(
      reportPreparation: ({required input}) {
        sent.add(input);
        return sent.length == 1
            ? first.future
            : Future.value(FfiPlayerPreparationDisposition.applied);
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

    first.complete(FfiPlayerPreparationDisposition.applied);
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

  test('six in-flight reporters bound controller tracking', () async {
    final blocked = <Completer<FfiPlayerPreparationDisposition>>[];
    final feedback = FfiPlayerPreparationFeedbackPort(
      reportPreparation: ({required input}) {
        final completion = Completer<FfiPlayerPreparationDisposition>();
        blocked.add(completion);
        return completion.future;
      },
      playerCapabilityGeneration: BigInt.one,
      clientEpoch: BigInt.one,
      monotonicMicros: () => 1,
    );

    for (var index = 0; index < 7; index += 1) {
      feedback.prepare(testPlaybackAuthority(postId: 'post-$index')).begin();
    }
    expect(blocked, hasLength(6));
    for (final completion in blocked) {
      completion.complete(FfiPlayerPreparationDisposition.applied);
    }
    await drainTestMicrotasks(12);
  });
}
