import 'dart:async';

import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/async/parallel_wait.dart';

void main() {
  test('surfaces the first failure while the peer remains pending', () {
    fakeAsync((async) {
      final stalled = Completer<String>();
      final failure = StateError('first failed');
      Object? observedError;

      unawaited(
        waitForBoth<int, String>(
          Future<int>.error(failure),
          stalled.future,
        ).then<void>(
          (_) => fail('The failed pair unexpectedly completed.'),
          onError: (Object error, StackTrace _) => observedError = error,
        ),
      );

      try {
        async.flushMicrotasks();
        expect(observedError, same(failure));
      } finally {
        stalled.complete('released');
        async.flushMicrotasks();
      }
    });
  });
}
