import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/work/retrieval_scheduler.dart';

void main() {
  test('a failing request surfaces its error and frees the worker', () async {
    final scheduler = RetrievalScheduler(maxConcurrent: 1);

    final failing = scheduler.run<int>(
      const RetrievalRequest(context: 'feed'),
      () async => throw StateError('relay down'),
    );
    final following = scheduler.run(
      const RetrievalRequest(context: 'feed'),
      () async => 42,
    );

    await expectLater(failing, throwsStateError);
    expect(await following, 42);
  });
}
