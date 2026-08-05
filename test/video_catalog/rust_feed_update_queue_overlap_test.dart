import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_update_queue.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

import '../support/rust_feed_fixtures.dart';

void main() {
  test('one full snapshot releases every overlapping reader', () async {
    final updates = StreamController<FfiFeedUpdate>();
    final queue = RustFeedUpdateQueue(updates.stream);
    const wait = Duration(seconds: 1);

    final first = queue.next(wait);
    final second = queue.next(wait);
    updates.add(rustFeedUpdate(revision: 7));

    expect((await first)?.revision, BigInt.from(7));
    expect((await second)?.revision, BigInt.from(7));
    await queue.dispose();
  });
}
