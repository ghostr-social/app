part of 'app_controller_factory.dart';

extension AppControllerFeedFactories on AppControllerFactory {
  FeedCubit feed({ProfileId? viewerId}) {
    final services = _dependencies.videoCatalogServices;
    return FeedCubit(
      _feedDependencies(
        services.feed,
        viewerId: viewerId,
        updates: services.feedUpdates,
      ),
    );
  }

  FeedCubit discoveryFeed(String query, {ProfileId? viewerId}) {
    return FeedCubit(
      _feedDependencies(
        QueryVideoFeedRepository(
          search: _dependencies.videoCatalogServices.search,
          query: query,
        ),
        viewerId: viewerId,
      ),
    );
  }

  FeedCubit profileFeed(ProfileSummary viewer, VideoPost post) {
    return FeedCubit(
      _feedDependencies(
        ProfileVideoFeedRepository(
          profile: _dependencies.videoCatalogServices.profile,
          viewer: viewer,
          creatorId: post.creator.id,
        ),
        viewerId: viewer.id,
      ),
      openAt: post.id,
    );
  }

  FeedDependencies _feedDependencies(
    VideoFeedRepository feed, {
    ProfileId? viewerId,
    VideoFeedUpdates? updates,
  }) {
    final services = _dependencies.videoCatalogServices;
    return FeedDependencies(
      viewerId: viewerId,
      feed: ensureRepostHydratedVideoFeed(feed, services.reposts),
      engagement: services.engagement,
      followProfile: _followWorkflow(services.social),
      optional: FeedOptionalDependencies(
        social: services.social,
        focus: _feedFocus,
        watchTracker: _watchTracker(),
        delivery: FeedDeliveryDependencies(
          updates: updates,
          reposts: services.reposts,
        ),
      ),
    );
  }

  DefaultFollowProfileWorkflow _followWorkflow(SocialGraphRepository social) {
    return DefaultFollowProfileWorkflow(
      social: social,
      activity: _dependencies.activityRepository,
      clock: DateTime.now,
      failureReporter: _dependencies.failureReporter,
    );
  }

  WatchHistoryTracker _watchTracker() {
    return WatchHistoryTracker(
      history: _dependencies.watchHistoryRepository,
      settings: _dependencies.appSettingsRepository,
      failureReporter: _dependencies.failureReporter,
    );
  }
}
