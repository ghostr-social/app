import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_sha256.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';

import '../support/sample_data.dart';

void main() {
  test('a watch entry remembers the video URL and file digest', () {
    final digest = VideoSha256.tryParse('a' * 64)!;
    final post = samplePost(id: 'clip');
    final hashed = post.withMedia(
      VideoMediaSource.withExpectedSha256(post.media, digest.value),
    );

    final entry = WatchHistoryEntry.fromPost(hashed, DateTime.utc(2026, 8, 3));

    expect(entry.mediaUrl, 'https://example.com/video/clip.mp4');
    expect(entry.mediaSha256, 'a' * 64);

    final local = post.withMedia(VideoMediaSource.local('/videos/draft.mp4'));
    final localEntry =
        WatchHistoryEntry.fromPost(local, DateTime.utc(2026, 8, 3));
    expect(localEntry.mediaUrl, isNull);
    expect(localEntry.mediaSha256, isNull);
  });
}
