import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_id.dart';

VideoPost progressiveDevicePost({
  required String socialId,
  required String deliveryId,
  required Uri origin,
}) {
  final media = VideoMediaSource.withCacheScope(
    VideoMediaSource.remote(origin.toString()),
    deliveryId,
  );
  return VideoPost(
    identity: VideoPostIdentity(
      id: VideoPostId.parse(socialId),
      creator: _creator,
    ),
    content: VideoPostContent(
      caption: socialId,
      songName: 'Device fixture',
      media: media,
      publishedAt: DateTime.utc(2026, 8, 14),
    ),
    metrics: VideoPostMetrics(
      likeCount: 0,
      commentCount: 0,
      viewerHasLiked: false,
    ),
  );
}

final _creator = ProfileSummary(
  id: ProfileId.parse('device-fixture-creator'),
  displayName: 'Device Fixture',
  handle: '@fixture',
  avatarUrl: null,
);
