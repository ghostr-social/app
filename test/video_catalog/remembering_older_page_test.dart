import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_snapshot.dart';
import 'package:ghostr/features/video_catalog/data/remembering_remote_video_source.dart';

import '../support/fake_remote_video_source.dart';
import '../support/nostr_reference.dart';
import '../support/sample_data.dart';

void main() {
  test('an older page never becomes the canonical snapshot', () async {
    final snapshot = NostrVideoSnapshot();
    final remote = FakeRemoteVideoSource([
      samplePost(id: 'newest-1', nostrReference: nostrReference()),
    ])
      ..olderPosts = [
        samplePost(
          id: 'older-1',
          nostrReference: nostrReference(eventId: secondTestEventId),
        ),
      ];
    final source = RememberingRemoteVideoSource(remote, snapshot);
    await source.loadRemoteFeed();

    final page = await source.loadRemoteFeed(
      olderThan: DateTime.utc(2026, 8, 1),
    );

    expect(page.map((post) => post.id.value), ['older-1']);
    expect(snapshot.read().map((post) => post.id.value), ['newest-1']);
  });
}
