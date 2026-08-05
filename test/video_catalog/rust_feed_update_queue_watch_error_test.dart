import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_update_queue.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

void main() {
  test('active and late feed watchers receive a native stream failure',
      () async {
    final upstream = StreamController<FfiFeedUpdate>(sync: true);
    final queue = RustFeedUpdateQueue(upstream.stream);
    final activeErrors = <Object>[];
    final activeDone = Completer<void>();
    final active = queue.watch().listen(
          (_) {},
          onError: activeErrors.add,
          onDone: activeDone.complete,
        );

    upstream.addError(StateError('watcher died'));
    await activeDone.future;

    expect(activeErrors.single, isA<StateError>());
    await expectLater(queue.watch(), emitsError(isA<StateError>()));
    await active.cancel();
    await queue.dispose();
    await upstream.close();
  });
}
