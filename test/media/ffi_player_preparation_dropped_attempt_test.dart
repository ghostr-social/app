import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/ffi_player_preparation_feedback_port.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';

import '../support/drain_test_microtasks.dart';
import '../support/playback_authority_fixture.dart';

void main() {
  test('a released controller cannot silence its replacement', () async {
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
    final retired = feedback.prepare(testPlaybackAuthority(postId: 'retired'))
      ..begin();
    for (var index = 1; index < 8; index += 1) {
      feedback.prepare(testPlaybackAuthority(postId: 'active-$index')).begin();
    }
    retired.release();
    feedback.prepare(testPlaybackAuthority(postId: 'replacement')).begin();
    await drainTestMicrotasks();

    expect(
      sent.where((report) => report.postId == 'replacement'),
      hasLength(1),
    );
    blocked.first.complete(FfiPlayerPreparationDisposition.applied);
    await drainTestMicrotasks();
    expect(
      sent
          .where((report) => report.postId == 'retired')
          .map((report) => report.state),
      [
        FfiPlayerPreparationState.initializing,
        FfiPlayerPreparationState.released,
      ],
    );
    for (final completion in blocked.where((value) => !value.isCompleted)) {
      completion.complete(FfiPlayerPreparationDisposition.applied);
    }
    await drainTestMicrotasks();
  });
}
