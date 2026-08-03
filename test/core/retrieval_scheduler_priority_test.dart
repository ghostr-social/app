import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/work/retrieval_scheduler.dart';

void main() {
  test('interactive work starts before queued background work', () async {
    final scheduler = RetrievalScheduler(maxConcurrent: 1);
    final gate = Completer<void>();
    final started = <String>[];

    unawaited(scheduler.run(
      const RetrievalRequest(context: 'feed'),
      () => gate.future,
    ));
    final background = scheduler.run(
      const RetrievalRequest(
        context: 'digging',
        priority: RetrievalPriority.background,
      ),
      () async => started.add('background'),
    );
    final enrichment = scheduler.run(
      const RetrievalRequest(
        context: 'likes',
        priority: RetrievalPriority.enrichment,
      ),
      () async => started.add('enrichment'),
    );
    final interactive = scheduler.run(
      const RetrievalRequest(context: 'search:ghost'),
      () async => started.add('interactive'),
    );

    gate.complete();
    await Future.wait([background, enrichment, interactive]);
    expect(started, ['interactive', 'enrichment', 'background']);
  });
}
