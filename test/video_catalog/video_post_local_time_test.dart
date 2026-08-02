import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_id.dart';

import '../support/sample_data.dart';

void main() {
  test('uses the explicitly supplied publication time for a local post', () {
    final publishedAt = DateTime.utc(2026, 8, 2, 12);

    final post = VideoPost(
      identity: VideoPostIdentity(
        id: VideoPostId.parse('local-1'),
        creator: sampleCreator(),
      ),
      content: VideoPostContent(
        caption: 'Local clip',
        songName: 'Original sound',
        media: VideoMediaSource.local('/tmp/video.mp4'),
        publishedAt: publishedAt,
      ),
      metrics: VideoPostMetrics(
        likeCount: 0,
        commentCount: 0,
        viewerHasLiked: false,
      ),
    );

    expect(post.publishedAt, publishedAt);
  });
}
