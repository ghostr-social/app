import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/data/ffi_rust_feed_port.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_identity.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

import '../support/nostr_test_values.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  test('forwards every call to the generated bridge functions', () async {
    final calls = <String>[];
    final update = rustFeedUpdate(revision: 3);
    final port = FfiRustFeedPort(
      session: ({expectedAccountHex}) async {
        calls.add('session:$expectedAccountHex');
        return BigInt.from(4);
      },
      open: ({
        required spec,
        expectedAccountHex,
        required expectedSessionGeneration,
      }) async {
        calls.add(
          'open:${spec.kind.name}:$expectedAccountHex:'
          '$expectedSessionGeneration',
        );
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

    final account = NostrPublicKeyHex.parse(testViewerPublicKey);
    final session = await port.captureSession(account);
    final feedId = await port.openFeed(
      const FfiFeedSpec(
        kind: FfiFeedKind.search,
        value: 'ghost',
        creators: [],
      ),
      session,
    );
    final seen = await port.feedUpdates(feedId).first;
    final more = await port.loadMore(feedId, olderThanSecs: BigInt.two);
    await port.closeFeed(feedId);

    expect(feedId, RustFeedId.parse('9'));
    expect(seen, update);
    expect(more, isTrue);
    expect(calls, [
      'session:$testViewerPublicKey',
      'open:search:$testViewerPublicKey:4',
      'watch:9',
      'more:9:2',
      'close:9',
    ]);
  });
}
