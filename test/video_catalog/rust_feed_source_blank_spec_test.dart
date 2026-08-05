import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/fake_rust_feed_port.dart';

void main() {
  test('an empty creator scope never opens a native feed', () async {
    final port = FakeRustFeedPort();
    final source = RustFeedRemoteSource(port: port);
    final creators = <ProfileId>{};

    expect(await source.loadRemoteFeed(creatorIds: creators), isEmpty);
    expect(await source.loadMoreRemoteFeed(creatorIds: creators), isEmpty);
    await expectLater(
      source.watchRemoteFeed(creatorIds: creators),
      emitsDone,
    );
    expect(port.openedSpecs, isEmpty);
  });
}
