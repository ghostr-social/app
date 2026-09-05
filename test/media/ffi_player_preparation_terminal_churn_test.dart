import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';

import '../support/drain_test_microtasks.dart';
import '../support/ffi_preparation_feedback_fixture.dart';
import '../support/playback_authority_fixture.dart';

void main() {
  test('a dispatched initial keeps its queued terminal under churn', () async {
    final first = Completer<FfiPlayerPreparationDisposition>();
    final sent = <FfiPlayerPreparationReport>[];
    final feedback = preparationFeedback(({required input}) {
      sent.add(input);
      return sent.length == 1
          ? first.future
          : Future.value(FfiPlayerPreparationDisposition.applied);
    });
    final active = feedback.prepare(testPlaybackAuthority(postId: 'active'));
    active.begin();
    active.release();
    for (var index = 0; index < 8; index += 1) {
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
}
