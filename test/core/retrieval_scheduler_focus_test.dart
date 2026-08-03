import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/work/retrieval_scheduler.dart';

void main() {
  test('focusing a context moves its queued work ahead of everything', () async {
    final scheduler = RetrievalScheduler(maxConcurrent: 1);
    final gate = Completer<void>();
    final started = <String>[];

    unawaited(scheduler.run(
      const RetrievalRequest(context: 'feed'),
      () => gate.future,
    ));
    final stale = scheduler.run(
      const RetrievalRequest(context: 'search:old'),
      () async => started.add('stale-search'),
    );
    final focused = scheduler.run(
      const RetrievalRequest(
        context: 'tag:dance',
        priority: RetrievalPriority.background,
      ),
      () async => started.add('focused-digging'),
    );

    scheduler.focus('tag:dance');
    gate.complete();
    await Future.wait([stale, focused]);

    // The focused context wins even against higher-priority stale work.
    expect(started, ['focused-digging', 'stale-search']);
  });
}
