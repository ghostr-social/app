import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/ffi_player_preparation_feedback_port.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';

import '../support/drain_test_microtasks.dart';
import '../support/playback_authority_fixture.dart';

void main() {
  test('one stalled controller generation retains one replacement', () async {
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
    final retired = List.generate(6, (index) {
      return feedback.prepare(testPlaybackAuthority(postId: 'retired-$index'))
        ..begin();
    });
    for (final attempt in retired) {
      attempt.release();
    }
    final replacements = List.generate(6, (index) {
      return feedback.prepare(
        testPlaybackAuthority(postId: 'replacement-$index'),
      )..begin();
    });
    replacements.first.release();
    final overflow = feedback.prepare(testPlaybackAuthority(postId: 'overflow'))
      ..begin();
    overflow.release();
    await drainTestMicrotasks();

    expect(sent, hasLength(12));
    expect(sent.where((report) => report.postId == 'overflow'), isEmpty);
    for (final completion in blocked.take(6)) {
      completion.complete(FfiPlayerPreparationDisposition.applied);
    }
    await drainTestMicrotasks();
    expect(
      sent.where(
        (report) => report.state == FfiPlayerPreparationState.released,
      ),
      hasLength(6),
    );
    blocked[12].complete(FfiPlayerPreparationDisposition.applied);
    await drainTestMicrotasks();
    feedback.prepare(testPlaybackAuthority(postId: 'reopened')).begin();
    await drainTestMicrotasks();
    expect(sent.where((report) => report.postId == 'reopened'), hasLength(1));
    for (final completion in blocked.where((value) => !value.isCompleted)) {
      completion.complete(FfiPlayerPreparationDisposition.applied);
    }
    await drainTestMicrotasks();
  });
}
