import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_snapshot.dart';
import 'package:ghostr/features/video_catalog/data/remembering_remote_video_source.dart';

import '../support/fakes.dart';
import '../support/nostr_test_values.dart';
import '../support/nostr_video_post_fixture.dart';

void main() {
  test('a hashtag-scoped load cannot become the canonical snapshot', () async {
    final post = nostrVideoPost(const NostrVideoPostFixture(
      eventId: testEventId,
      mediaId: 'scoped',
    ));
    final snapshot = NostrVideoSnapshot();
    final source = RememberingRemoteVideoSource(
      FakeRemoteVideoSource([post]),
      snapshot,
    );

    await source.loadRemoteFeed(hashtags: {'dance'});

    expect(snapshot.read(), isEmpty);
  });
}
