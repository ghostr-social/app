import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watched_video_index.dart';

import '../support/sample_data.dart';

void main() {
  test('a video counts as watched by id, by URL, or by file digest', () {
    final watchedAt = DateTime.utc(2026, 8, 1);
    final index = WatchedVideoIndex([
      WatchHistoryEntry(
        videoId: samplePost(id: 'seen').id.value,
        title: 'Seen',
        creatorName: 'Nora',
        watchedAt: watchedAt,
        mediaUrl: 'https://example.com/video/seen.mp4',
        mediaSha256: 'c' * 64,
      ),
    ]);

    expect(index.isEmpty, isFalse);
    expect(index.watchedAt(samplePost(id: 'seen')), watchedAt);

    // A republish under a new event id but the same URL stays hidden.
    final repost = samplePost(id: 'repost').withMedia(
      VideoMediaSource.remote('https://example.com/video/seen.mp4'),
    );
    expect(index.watchedAt(repost), watchedAt);

    // Same file on a different host is caught by its digest.
    final mirrored = samplePost(id: 'mirror').withMedia(
      VideoMediaSource.withExpectedSha256(
        VideoMediaSource.remote('https://mirror.example/other.mp4'),
        'c' * 64,
      ),
    );
    expect(index.watchedAt(mirrored), watchedAt);

    expect(index.watchedAt(samplePost(id: 'unseen')), isNull);
    expect(WatchedVideoIndex(const []).isEmpty, isTrue);
  });
}
