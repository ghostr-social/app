import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_loads.dart';

void main() {
  test('only the newest request may still change the feed', () {
    final loads = FeedLoads();

    final older = loads.take();
    expect(loads.accepts(older), isTrue);

    final newest = loads.take();
    expect(loads.accepts(older), isFalse);
    expect(loads.accepts(newest), isTrue);
    expect(loads.pending, newest);
  });

  test('an answer overtaken while it travelled comes back empty', () async {
    final loads = FeedLoads();
    final overtaken = Completer<String>();

    final answer = loads.newest(() => overtaken.future);
    loads.take();
    overtaken.complete('stale');

    expect(await answer, isNull);
    expect(await loads.newest(() async => 'fresh'), 'fresh');
  });
}
