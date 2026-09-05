import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';

import '../support/drain_test_microtasks.dart';
import '../support/ffi_preparation_feedback_fixture.dart';
import '../support/playback_authority_fixture.dart';

void main() {
  test('two in-flight reporters bound controller tracking', () async {
    final blocked = <Completer<FfiPlayerPreparationDisposition>>[];
    final feedback = preparationFeedback(({required input}) {
      final completion = Completer<FfiPlayerPreparationDisposition>();
      blocked.add(completion);
      return completion.future;
    });

    for (var index = 0; index < 9; index += 1) {
      feedback.prepare(testPlaybackAuthority(postId: 'post-$index')).begin();
    }
    expect(blocked, hasLength(2));
    for (final completion in blocked) {
      completion.complete(FfiPlayerPreparationDisposition.applied);
    }
    await drainTestMicrotasks(12);
  });
}
