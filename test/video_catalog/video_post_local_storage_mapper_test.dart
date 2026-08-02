import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/data/video_post_storage_mapper.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/sample_data.dart';

void main() {
  test('round trips a local post without a Nostr reference', () {
    final sample = samplePost();
    final post = VideoPost(
      identity: VideoPostIdentity(id: sample.id, creator: sample.creator),
      content: VideoPostContent(
        caption: sample.caption,
        songName: sample.songName,
        media: VideoMediaSource.local('/tmp/video.mp4'),
        publishedAt: sample.publishedAt,
      ),
      metrics: VideoPostMetrics(
        likeCount: 0,
        commentCount: 0,
        viewerHasLiked: false,
      ),
    );

    final decoded = const VideoPostStorageMapper().fromMap(
      const VideoPostStorageMapper().toMap(post),
    );

    expect(decoded.media.localPath, '/tmp/video.mp4');
    expect(decoded.nostrReference, isNull);
    expect(decoded.viewerHasLiked, isFalse);
  });
}
