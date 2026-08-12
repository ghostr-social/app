import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watched_video_index.dart';

import '../support/sample_data.dart';

void main() {
  test('a watched URL served as a mirror fallback still counts as watched',
      () {
    final watched = samplePost(id: 'original').withMedia(
      VideoMediaSource.remote('https://mirror.example/clip.mp4'),
    );
    final index = WatchedVideoIndex([
      WatchHistoryEntry.fromPost(watched, DateTime.utc(2026, 8, 1)),
    ]);
    final republished = samplePost(id: 'republished').withMedia(
      VideoMediaSource.remote(
        'https://primary.example/clip.mp4',
        fallbackUrls: ['https://mirror.example/clip.mp4'],
      ),
    );

    expect(index.contains(republished), isTrue);
  });
}
