import 'package:ghostr/app/app_dependencies.dart';
import 'package:ghostr/core/media/incoming_video_share.dart';
import 'package:ghostr/features/activity/presentation/activity_cubit.dart';
import 'package:ghostr/features/comments/presentation/comments_cubit.dart';
import 'package:ghostr/features/profile/presentation/profile_edit_cubit.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';
import 'package:ghostr/features/compose/presentation/compose_cubit.dart';
import 'package:ghostr/features/compose/domain/publish_video_workflow.dart';
import 'package:ghostr/features/settings/presentation/settings_cubit.dart';
import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/social/presentation/blocked_accounts_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/profile_video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/query_video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/repost_hydrated_video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/trending_hashtags_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/toggle_profile_follow_workflow.dart';
import 'package:ghostr/features/video_catalog/domain/follow_profile_workflow.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_arbiter.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_feed_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_search_repository.dart';
import 'package:ghostr/features/watch_history/presentation/watch_history_cubit.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_workflow.dart';
import 'package:ghostr/platform/media/delivery_config_syncing_settings_repository.dart';
import 'package:ghostr/platform/media/ffi_feed_focus_port.dart';
import 'package:ghostr/platform/media/ffi_video_delivery_updates.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

part 'app_controller_factory_feed.dart';
part 'app_controller_factory_media.dart';

class AppControllerFactory {
  AppControllerFactory(
    this._dependencies, {
    FeedFocusPort? feedFocus,
    VideoDeliveryUpdates? deliveryUpdates,
    RustDeliveryConfigUpdater deliveryConfigUpdater =
        updateRustEngineConfiguration,
  }) : _feedFocus = FeedFocusArbiter(feedFocus ?? FfiFeedFocusPort()),
       _deliveryUpdates = deliveryUpdates ?? FfiVideoDeliveryUpdates(),
       _deliveryConfigUpdater = deliveryConfigUpdater;

  final AppDependencies _dependencies;
  final FeedFocusArbiter _feedFocus;
  final VideoDeliveryUpdates _deliveryUpdates;
  final RustDeliveryConfigUpdater _deliveryConfigUpdater;
  ActivityCubit activity() => ActivityCubit(
    _dependencies.activityRepository.snapshotForActiveAccount(),
  );
  SearchCubit search() {
    final services = _dependencies.videoCatalogServices;
    final search = WatchAwareVideoSearchRepository(
      search: services.search,
      updates: services.searchUpdates,
      history: _dependencies.watchHistoryRepository,
      failureReporter: _dependencies.failureReporter,
    );
    return SearchCubit(search, updates: search);
  }

  TrendingHashtagsCubit trending() {
    return TrendingHashtagsCubit(_dependencies.videoCatalogServices.trending);
  }

  WatchHistoryCubit watchHistory() {
    return WatchHistoryCubit(
      _dependencies.watchHistoryRepository.snapshotForActiveAccount(),
    );
  }

  BlockedAccountsCubit blockedAccounts() {
    return BlockedAccountsCubit(
      _dependencies.videoCatalogServices.social,
      _dependencies.profileMetadataRepository,
    );
  }

  CommentsCubit comments(VideoPost post) {
    return CommentsCubit(_dependencies.videoCatalogServices.comments, post);
  }

  ComposeCubit compose() {
    return ComposeCubit(
      ComposeDependencies(
        publishVideo: DefaultPublishVideoWorkflow(
          publishing: _dependencies.videoCatalogServices.publishing,
          activity: _dependencies.activityRepository,
          clock: DateTime.now,
          failureReporter: _dependencies.failureReporter,
        ),
        mediaPicker: _dependencies.mediaPickerPort,
      ),
    );
  }

  ProfileCubit profile(
    ProfileSummary viewer,
    ProfileId profileId, {
    void Function(ProfileSummary)? onCurrentProfileUpdated,
  }) {
    return ProfileCubit(
      ProfileDependencies(
        profile: _dependencies.videoCatalogServices.profile,
        metadata: _dependencies.profileMetadataRepository,
        onCurrentProfileUpdated: onCurrentProfileUpdated,
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

  ProfileEditCubit profileEdit(NostrIdentity identity) {
    return ProfileEditCubit(
      _dependencies.profileMetadataRepository,
      identity,
      _dependencies.profileImageWorkflow,
    );
  }

  SettingsCubit settings() {
    return SettingsCubit(
      DeliveryConfigSyncingSettingsRepository(
        inner: _dependencies.appSettingsRepository,
        updateConfig: _deliveryConfigUpdater,
      ),
    );
  }
}
