import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';

import '../support/drain_test_microtasks.dart';
import '../support/ffi_preparation_feedback_fixture.dart';
import '../support/playback_authority_fixture.dart';

void main() {
  test('both controller attempts survive a blocked reporter', () async {
    final first = Completer<FfiPlayerPreparationDisposition>();
    final sent = <FfiPlayerPreparationReport>[];
    final feedback = preparationFeedback(({required input}) {
      sent.add(input);
      return sent.length == 1
          ? first.future
          : Future.value(FfiPlayerPreparationDisposition.applied);
    });

    for (var index = 0; index < 2; index += 1) {
      feedback.prepare(testPlaybackAuthority(postId: 'post-$index')).begin();
    }
    await drainTestMicrotasks();
    expect(sent, hasLength(2));
    first.complete(FfiPlayerPreparationDisposition.applied);
    await drainTestMicrotasks(12);

    expect(sent, hasLength(2));
    expect(sent.last.postId, 'post-1');
  });
}
