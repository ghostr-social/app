import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watched_video_index.dart';

import '../support/sample_data.dart';

const _digest =
    'f2ca1bb6c7e907d06dafe4687e579fce76b37e4e93b7605022da52e6ccc26fd2';

void main() {
  test('the same file on another blossom host still counts as watched', () {
    final watched = samplePost(id: 'original').withMedia(
      VideoMediaSource.remote('https://host-a.example/$_digest.mp4'),
    );
    final index = WatchedVideoIndex([
      WatchHistoryEntry.fromPost(watched, DateTime.utc(2026, 8, 1)),
    ]);
    final republished = samplePost(id: 'republished').withMedia(
      VideoMediaSource.remote('https://host-b.example/$_digest.mp4'),
    );

    expect(index.contains(republished), isTrue);
    expect(index.contains(samplePost(id: 'unrelated')), isFalse);
  });
}
