import 'package:ghostr/features/comments/domain/video_comments_repository.dart';
import 'package:ghostr/features/engagement/domain/video_engagement_repository.dart';
import 'package:ghostr/features/publish/domain/video_publishing_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_profile_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_repository.dart';

class VideoCatalogServices {
  const VideoCatalogServices({
    required this.feed,
    required this.engagement,
    required this.profile,
    required this.search,
    required this.publishing,
    required this.comments,
  });

  final VideoFeedRepository feed;
  final VideoEngagementRepository engagement;
  final VideoProfileRepository profile;
  final VideoSearchRepository search;
  final VideoPublishingRepository publishing;
  final VideoCommentsRepository comments;
}
