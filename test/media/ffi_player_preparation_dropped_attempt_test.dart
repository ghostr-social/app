import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/ffi_player_preparation_feedback_port.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';

import '../support/drain_test_microtasks.dart';
import '../support/playback_authority_fixture.dart';

void main() {
  test('an undispatched attempt stays silent after bounded eviction', () async {
    final blocked = Completer<void>();
    final sent = <FfiPlayerPreparationReport>[];
    final feedback = FfiPlayerPreparationFeedbackPort(
      reportPreparation: ({required input}) {
        sent.add(input);
        return sent.length == 1 ? blocked.future : Future.value();
      },
      playerCapabilityGeneration: BigInt.one,
      clientEpoch: BigInt.one,
      monotonicMicros: () => 1,
    );
    final active = feedback.prepare(testPlaybackAuthority(postId: 'active'));
    active.begin();
    active.release();
    final dropped = feedback.prepare(testPlaybackAuthority(postId: 'dropped'));
    dropped.begin();
    for (var index = 0; index < 5; index += 1) {
      feedback.prepare(testPlaybackAuthority(postId: 'churn-$index')).begin();
    }
    dropped.release();

    blocked.complete();
    await drainTestMicrotasks(20);

    expect(sent.where((report) => report.postId == 'dropped'), isEmpty);
  });
}
