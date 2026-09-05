import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';

import '../support/drain_test_microtasks.dart';
import '../support/ffi_preparation_feedback_fixture.dart';
import '../support/playback_authority_fixture.dart';

void main() {
  test('one rendered preparation admits the next initializer', () async {
    final sent = <FfiPlayerPreparationReport>[];
    final feedback = preparationFeedback(({required input}) async {
      sent.add(input);
      return FfiPlayerPreparationDisposition.applied;
    });

    for (var index = 0; index < 1; index += 1) {
      final attempt = feedback.prepare(
        testPlaybackAuthority(postId: 'ready-$index'),
      );
      attempt.begin();
      attempt.initialized();
      attempt.firstFrameRendered();
    }
    feedback.prepare(testPlaybackAuthority(postId: 'new')).begin();
    await drainTestMicrotasks(20);

    expect(
      sent.where(
        (report) =>
            report.postId == 'new' &&
            report.state == FfiPlayerPreparationState.initializing,
      ),
      hasLength(1),
    );
    expect(
      sent.where(
        (report) => report.state == FfiPlayerPreparationState.released,
      ),
      isEmpty,
    );
  });
}
