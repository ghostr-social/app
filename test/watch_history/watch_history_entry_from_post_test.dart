import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';

import '../support/nostr_test_values.dart';
import '../support/nostr_video_post_fixture.dart';
import '../support/sample_data.dart';

void main() {
  test('builds the watch entry identity and title from the post', () {
    final watchedAt = DateTime.utc(2026, 3, 12, 10);

    final plain = WatchHistoryEntry.fromPost(samplePost(), watchedAt);
    expect(plain.videoId, 'e:post-1');
    expect(plain.title, 'A relay-side banger');
    expect(plain.creatorName, 'Nora Relay');

    final addressable = WatchHistoryEntry.fromPost(
      nostrVideoPost(
        NostrVideoPostFixture(eventId: testEventId, mediaId: 'media-1'),
      ),
      watchedAt,
    );
    expect(addressable.videoId, 'a:34235:$testCreatorPublicKey:media-1');

    final untitled = WatchHistoryEntry.fromPost(
      samplePost(caption: '   '),
      watchedAt,
    );
    expect(untitled.title, 'Original sound');
  });
}
