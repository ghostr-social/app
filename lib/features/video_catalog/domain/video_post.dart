import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_id.dart';

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
  });

  final String caption;
  final String songName;
  final VideoMediaSource media;
  final DateTime publishedAt;
}

class VideoPostMetrics {
  factory VideoPostMetrics({
    required int likeCount,
    required int commentCount,
    required bool viewerHasLiked,
  }) {
    _checkCount(likeCount, 'likeCount');
    _checkCount(commentCount, 'commentCount');
    return VideoPostMetrics._(likeCount, commentCount, viewerHasLiked);
  }

  const VideoPostMetrics._(
    this.likeCount,
    this.commentCount,
    this.viewerHasLiked,
  );

  final int likeCount;
  final int commentCount;
  final bool viewerHasLiked;
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
  VideoMediaSource get media => content.media;
  DateTime get publishedAt => content.publishedAt;
  int get likeCount => metrics.likeCount;
  int get commentCount => metrics.commentCount;
  bool get viewerHasLiked => metrics.viewerHasLiked;

  VideoPost withInteraction({
    required int likeCount,
    required bool viewerHasLiked,
    int? commentCount,
  }) {
    return VideoPost(
      identity: identity,
      content: content,
      metrics: VideoPostMetrics(
        likeCount: likeCount,
        commentCount: commentCount ?? this.commentCount,
        viewerHasLiked: viewerHasLiked,
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
      ),
      metrics: metrics,
    );
  }
}

void _checkCount(int count, String name) {
  if (count < 0) throw RangeError.value(count, name, 'Cannot be negative.');
}
