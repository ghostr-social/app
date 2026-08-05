import 'package:ghostr/features/comments/domain/video_comments_repository.dart';
import 'package:ghostr/features/engagement/domain/video_engagement_repository.dart';
import 'package:ghostr/features/publish/domain/video_publishing_repository.dart';
import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/trending_hashtags.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_profile_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_updates.dart';

class VideoCatalogServices {
  const VideoCatalogServices({
    required this.feed,
    required this.engagement,
    required this.profile,
    required this.search,
    required this.searchUpdates,
    required this.trending,
    required this.publishing,
    required this.comments,
    required this.social,
  });

  final VideoFeedRepository feed;
  final VideoEngagementRepository engagement;
  final VideoProfileRepository profile;
  final VideoSearchRepository search;
  final VideoSearchUpdates searchUpdates;
  final TrendingHashtagsSource trending;
  final VideoPublishingRepository publishing;
  final VideoCommentsRepository comments;
  final SocialGraphRepository social;
}
