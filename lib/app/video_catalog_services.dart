import 'package:ghostr/app/video_feed_binding.dart';
import 'package:ghostr/features/comments/domain/video_comments_repository.dart';
import 'package:ghostr/features/engagement/domain/video_engagement_repository.dart';
import 'package:ghostr/features/publish/domain/video_publishing_repository.dart';
import 'package:ghostr/features/reposts/domain/video_repost_repository.dart';
import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/trending_hashtags.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_profile_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_updates.dart';

class VideoCatalogServices {
  const VideoCatalogServices({
    required VideoFeedBinding feed,
    required VideoCatalogDiscoveryServices discovery,
    required VideoCatalogInteractionServices interactions,
    required VideoCatalogAuthoringServices authoring,
  }) : _feed = feed,
       _discovery = discovery,
       _interactions = interactions,
       _authoring = authoring;

  final VideoFeedBinding _feed;
  final VideoCatalogDiscoveryServices _discovery;
  final VideoCatalogInteractionServices _interactions;
  final VideoCatalogAuthoringServices _authoring;

  VideoFeedRepository get feed => _feed.repository;
  VideoFeedUpdates? get feedUpdates => _feed.updates;
  VideoEngagementRepository get engagement => _interactions.engagement;
  VideoProfileRepository get profile => _discovery.profile;
  VideoSearchRepository get search => _discovery.search;
  VideoSearchUpdates get searchUpdates => _discovery.searchUpdates;
  TrendingHashtagsSource get trending => _discovery.trending;
  VideoPublishingRepository get publishing => _authoring.publishing;
  VideoCommentsRepository get comments => _interactions.comments;
  SocialGraphRepository get social => _interactions.social;
  VideoRepostRepository get reposts => _interactions.reposts;
}

final class VideoCatalogDiscoveryServices {
  const VideoCatalogDiscoveryServices({
    required this.profile,
    required this.search,
    required this.searchUpdates,
    required this.trending,
  });

  final VideoProfileRepository profile;
  final VideoSearchRepository search;
  final VideoSearchUpdates searchUpdates;
  final TrendingHashtagsSource trending;
}

final class VideoCatalogInteractionServices {
  const VideoCatalogInteractionServices({
    required this.engagement,
    required this.comments,
    required this.social,
    required this.reposts,
  });

  final VideoEngagementRepository engagement;
  final VideoCommentsRepository comments;
  final SocialGraphRepository social;
  final VideoRepostRepository reposts;
}

final class VideoCatalogAuthoringServices {
  const VideoCatalogAuthoringServices({required this.publishing});

  final VideoPublishingRepository publishing;
}
