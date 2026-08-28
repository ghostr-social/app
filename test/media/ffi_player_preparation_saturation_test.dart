import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/ffi_player_preparation_feedback_port.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';

import '../support/drain_test_microtasks.dart';
import '../support/playback_authority_fixture.dart';

void main() {
  test('eight saturated initials yield to a healthy ninth', () async {
    final sent = <FfiPlayerPreparationReport>[];
    final feedback = FfiPlayerPreparationFeedbackPort(
      reportPreparation: ({required input}) async {
        sent.add(input);
        return input.postId == 'healthy'
            ? FfiPlayerPreparationDisposition.closed
            : FfiPlayerPreparationDisposition.saturated;
      },
      playerCapabilityGeneration: BigInt.one,
      clientEpoch: BigInt.one,
      monotonicMicros: () => 1,
    );
    final first = feedback.prepare(testPlaybackAuthority(postId: 'full-0'))
      ..begin();
    for (var index = 1; index < 8; index += 1) {
      feedback.prepare(testPlaybackAuthority(postId: 'full-$index')).begin();
    }
    await drainTestMicrotasks();
    first.release();

    feedback.prepare(testPlaybackAuthority(postId: 'healthy')).begin();
    await drainTestMicrotasks();

    expect(sent.where((report) => report.postId == 'healthy'), hasLength(1));
  });

  test('saturated followup keeps its acknowledged actor history', () async {
    final blocked = <Completer<FfiPlayerPreparationDisposition>>[];
    final sent = <FfiPlayerPreparationReport>[];
    var followups = 0;
    final feedback = FfiPlayerPreparationFeedbackPort(
      reportPreparation: ({required input}) {
        sent.add(input);
        if (input.postId == 'protected') {
          if (input.state == FfiPlayerPreparationState.initializing) {
            return Future.value(FfiPlayerPreparationDisposition.applied);
          }
          followups += 1;
          return Future.value(
            followups == 1
                ? FfiPlayerPreparationDisposition.saturated
                : FfiPlayerPreparationDisposition.closed,
          );
        }
        final completion = Completer<FfiPlayerPreparationDisposition>();
        blocked.add(completion);
        return completion.future;
      },
      playerCapabilityGeneration: BigInt.one,
      clientEpoch: BigInt.one,
      monotonicMicros: () => 1,
    );
    final protected = feedback.prepare(
      testPlaybackAuthority(postId: 'protected'),
    )..begin();
    await drainTestMicrotasks();
    protected.initialized();
    await drainTestMicrotasks();
    for (var index = 0; index < 7; index += 1) {
      feedback.prepare(testPlaybackAuthority(postId: 'blocked-$index')).begin();
    }
    feedback.prepare(testPlaybackAuthority(postId: 'healthy')).begin();

    expect(sent.where((report) => report.postId == 'healthy'), isEmpty);
    for (final completion in blocked) {
      completion.complete(FfiPlayerPreparationDisposition.applied);
    }
    await Future<void>.delayed(const Duration(milliseconds: 20));
  });
}
