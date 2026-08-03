import 'package:ghostr/app/app_dependencies.dart';
import 'package:ghostr/features/activity/presentation/activity_cubit.dart';
import 'package:ghostr/features/comments/presentation/comments_cubit.dart';
import 'package:ghostr/features/compose/presentation/compose_cubit.dart';
import 'package:ghostr/features/compose/domain/publish_video_workflow.dart';
import 'package:ghostr/features/settings/presentation/settings_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/toggle_profile_follow_workflow.dart';
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

  FeedCubit feed() {
    return FeedCubit(FeedDependencies(
      feed: _dependencies.videoCatalogServices.feed,
      engagement: _dependencies.videoCatalogServices.engagement,
    ));
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
