import 'package:ghostr/app/app_dependencies.dart';
import 'package:ghostr/features/activity/presentation/activity_cubit.dart';
import 'package:ghostr/features/comments/presentation/comments_cubit.dart';
import 'package:ghostr/features/compose/presentation/compose_cubit.dart';
import 'package:ghostr/features/compose/domain/publish_video_workflow.dart';
import 'package:ghostr/features/settings/presentation/settings_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/query_video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/trending_hashtags_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/toggle_profile_follow_workflow.dart';
import 'package:ghostr/features/video_inventory/domain/feed_media_prefetcher.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';
import 'package:ghostr/features/watch_history/presentation/watch_history_cubit.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

class AppControllerFactory {
  const AppControllerFactory(this._dependencies);

  final AppDependencies _dependencies;

  ActivityCubit activity() {
    return ActivityCubit(
      _dependencies.activityRepository.snapshotForActiveAccount(),
    );
  }

  SearchCubit search() {
    return SearchCubit(_dependencies.videoCatalogServices.search);
  }

  TrendingHashtagsCubit trending() {
    return TrendingHashtagsCubit(_dependencies.videoCatalogServices.trending);
  }

  FeedCubit feed() {
    return FeedCubit(_feedDependencies(_dependencies.videoCatalogServices.feed));
  }

  /// A feed cubit bound to one search query or `#hashtag`.
  FeedCubit discoveryFeed(String query) {
    return FeedCubit(_feedDependencies(QueryVideoFeedRepository(
      search: _dependencies.videoCatalogServices.search,
      query: query,
    )));
  }

  FeedDependencies _feedDependencies(VideoFeedRepository feed) {
    return FeedDependencies(
      feed: feed,
      engagement: _dependencies.videoCatalogServices.engagement,
      social: _dependencies.videoCatalogServices.social,
      prefetcher: FeedMediaPrefetcher(inventory: _dependencies.videoInventory),
      watchTracker: WatchHistoryTracker(
        history: _dependencies.watchHistoryRepository,
        settings: _dependencies.appSettingsRepository,
        failureReporter: _dependencies.failureReporter,
      ),
    );
  }

  WatchHistoryCubit watchHistory() {
    return WatchHistoryCubit(
      _dependencies.watchHistoryRepository.snapshotForActiveAccount(),
    );
  }

  CommentsCubit comments(VideoPost post) {
    return CommentsCubit(_dependencies.videoCatalogServices.comments, post);
  }

  ComposeCubit compose() {
    return ComposeCubit(ComposeDependencies(
      publishVideo: DefaultPublishVideoWorkflow(
        publishing: _dependencies.videoCatalogServices.publishing,
        activity: _dependencies.activityRepository,
        clock: DateTime.now,
        failureReporter: _dependencies.failureReporter,
      ),
      mediaPicker: _dependencies.mediaPickerPort,
    ));
  }

  ProfileCubit profile(ProfileSummary viewer, ProfileId profileId) {
    return ProfileCubit(
      ProfileDependencies(
        profile: _dependencies.videoCatalogServices.profile,
        toggleFollow: DefaultToggleProfileFollowWorkflow(
          profile: _dependencies.videoCatalogServices.profile,
          activity: _dependencies.activityRepository,
          clock: DateTime.now,
          failureReporter: _dependencies.failureReporter,
        ),
      ),
      ProfileRequest(viewer: viewer, profileId: profileId),
    );
  }

  SettingsCubit settings() {
    return SettingsCubit(_dependencies.appSettingsRepository);
  }

  VideoPlaybackPort get videoPlaybackPort => _dependencies.videoPlaybackPort;
}
