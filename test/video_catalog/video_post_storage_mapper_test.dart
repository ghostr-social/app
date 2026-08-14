import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/data/video_post_storage_mapper.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/sample_data.dart';
import '../support/nostr_reference.dart';

void main() {
  test('round trips every persisted video-post field', () {
    const mapper = VideoPostStorageMapper();
    final sample = samplePost();
    final post = VideoPost(
      identity: VideoPostIdentity(
        id: sample.id,
        creator: sample.creator,
        nostrReference: nostrReferenceForAuthor(
          testAuthorPublicKey,
          eventId: testEventId,
          kind: 34236,
          identifier: 'clip-1',
        ),
      ),
      content: VideoPostContent(
        caption: sample.caption,
        songName: sample.songName,
        media: VideoMediaSource.remote(
          'https://primary.example/video.mp4',
          fallbackUrls: ['https://fallback.example/video.mp4'],
        ),
        publishedAt: sample.publishedAt,
      ),
      metrics: VideoPostMetrics(
        likeCount: sample.likeCount,
        commentCount: sample.commentCount,
        viewerHasLiked: sample.viewerHasLiked,
      ),
    );

    final decoded = mapper.fromMap(mapper.toMap(post));

    expect(decoded.caption, post.caption);
    expect(decoded.creator.displayName, post.creator.displayName);
    expect(decoded.media.remoteUrls, post.media.remoteUrls);
    expect(decoded.nostrReference?.eventId, testEventId);
    expect(decoded.publishedAt, post.publishedAt);
  });
}
