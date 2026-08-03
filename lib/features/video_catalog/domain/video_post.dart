import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_metrics.dart';

export 'package:ghostr/features/video_catalog/domain/video_post_metrics.dart';

class VideoPostIdentity {
  const VideoPostIdentity({
    required this.id,
    required this.creator,
    this.nostrReference,
  });

  final VideoPostId id;
  final ProfileSummary creator;
  final NostrEventReference? nostrReference;
}

class VideoPostContent {
  const VideoPostContent({
    required this.caption,
    required this.songName,
    required this.media,
    required this.publishedAt,
    this.hashtags = const <String>[],
  });

  final String caption;
  final String songName;
  final VideoMediaSource media;
  final DateTime publishedAt;
  final List<String> hashtags;
}

class VideoPost {
  const VideoPost({
    required this.identity,
    required this.content,
    required this.metrics,
  });

  final VideoPostIdentity identity;
  final VideoPostContent content;
  final VideoPostMetrics metrics;

  VideoPostId get id => identity.id;
  ProfileSummary get creator => identity.creator;
  NostrEventReference? get nostrReference => identity.nostrReference;
  String get caption => content.caption;
  String get songName => content.songName;
  List<String> get hashtags => content.hashtags;
  VideoMediaSource get media => content.media;
  DateTime get publishedAt => content.publishedAt;
  int get likeCount => metrics.likeCount;
  int get commentCount => metrics.commentCount;
  bool get viewerHasLiked => metrics.viewerHasLiked;

  VideoPost withInteraction(VideoInteractionUpdate update) {
    return VideoPost(
      identity: identity,
      content: content,
      metrics: VideoPostMetrics(
        likeCount: update.likeCount,
        commentCount: update.commentCount ?? commentCount,
        viewerHasLiked: update.viewerHasLiked,
        observations: metrics.observations.applying(update.observations),
      ),
    );
  }

  VideoPost withMedia(VideoMediaSource replacement) {
    return VideoPost(
      identity: identity,
      content: VideoPostContent(
        caption: caption,
        songName: songName,
        media: replacement,
        publishedAt: publishedAt,
        hashtags: hashtags,
      ),
      metrics: metrics,
    );
  }
}
