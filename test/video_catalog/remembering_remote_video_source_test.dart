import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_snapshot.dart';
import 'package:ghostr/features/video_catalog/data/remembering_remote_video_source.dart';

import '../support/fakes.dart';
import '../support/nostr_test_values.dart';
import '../support/nostr_video_post_fixture.dart';

void main() {
  test('retains the last non-empty canonical Nostr feed snapshot', () async {
    final post = nostrVideoPost(const NostrVideoPostFixture(
      eventId: testEventId,
      mediaId: 'remembered',
    ));
    final remote = FakeRemoteVideoSource([post]);
    final snapshot = NostrVideoSnapshot();
    final source = RememberingRemoteVideoSource(remote, snapshot);

    await source.loadRemoteFeed();
    remote.posts.clear();
    await source.loadRemoteFeed();

    expect(snapshot.read(), [post]);
  });
}
