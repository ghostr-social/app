import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_update_queue.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

void main() {
  test('a watcher failure reaches every overlapping reader', () async {
    final updates = StreamController<FfiFeedUpdate>();
    final queue = RustFeedUpdateQueue(updates.stream);
    const wait = Duration(seconds: 1);

    final first = queue.next(wait);
    final second = queue.next(wait);
    updates.addError(StateError('watcher died'));

    await expectLater(first, throwsA(isA<StateError>()));
    await expectLater(second, throwsA(isA<StateError>()));
    await queue.dispose();
  });
}
