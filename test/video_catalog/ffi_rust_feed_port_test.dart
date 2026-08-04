import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/ffi_rust_feed_port.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

import '../support/rust_feed_fixtures.dart';

void main() {
  test('forwards every call to the generated bridge functions', () async {
    final calls = <String>[];
    final update = rustFeedUpdate(revision: 3);
    final port = FfiRustFeedPort(
      open: ({required spec}) async {
        calls.add('open:${spec.kind}');
        return '9';
      },
      watch: ({required feedId}) {
        calls.add('watch:$feedId');
        return Stream.value(update);
      },
      more: ({required feedId, olderThanSecs}) async {
        calls.add('more:$feedId:$olderThanSecs');
        return true;
      },
      close: ({required feedId}) async => calls.add('close:$feedId'),
    );

    final feedId = await port.openFeed(
      const FfiFeedSpec(kind: 'search', value: 'ghost'),
    );
    final seen = await port.feedUpdates(feedId).first;
    final more = await port.loadMore(feedId, olderThanSecs: BigInt.two);
    await port.closeFeed(feedId);

    expect(feedId, '9');
    expect(seen, update);
    expect(more, isTrue);
    expect(calls, ['open:search', 'watch:9', 'more:9:2', 'close:9']);
  });
}
