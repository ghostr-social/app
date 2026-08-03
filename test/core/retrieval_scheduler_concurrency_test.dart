import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/work/retrieval_scheduler.dart';

void main() {
  test('runs at most the configured number of requests at once', () async {
    final scheduler = RetrievalScheduler(maxConcurrent: 2);
    final gates = List.generate(4, (_) => Completer<void>());
    final started = <int>[];

    final results = [
      for (final (index, gate) in gates.indexed)
        scheduler.run(const RetrievalRequest(context: 'feed'), () {
          started.add(index);
          return gate.future.then((_) => index);
        }),
    ];
    await Future<void>.delayed(Duration.zero);
    expect(started, [0, 1]);

    gates[0].complete();
    await Future<void>.delayed(Duration.zero);
    expect(started, [0, 1, 2]);

    for (final gate in gates.skip(1)) {
      gate.complete();
    }
    expect(await Future.wait(results), [0, 1, 2, 3]);
    expect(() => RetrievalScheduler(maxConcurrent: 0), throwsArgumentError);
  });
}
