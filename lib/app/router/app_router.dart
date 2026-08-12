import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/app/app_update_scope.dart';
import 'package:ghostr/app/profile_route_request.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:ghostr/features/profile/presentation/profile_edit_screen.dart';
import 'package:ghostr/features/settings/presentation/settings_screen.dart';
import 'package:ghostr/features/video_catalog/presentation/discovery_feed_screen.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_screen.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/watch_history/presentation/watch_history_screen.dart';

abstract final class AppRouter {
  /// A swipeable video feed for one search query or `#hashtag`.
  static Route<void> discoveryFeed(DiscoveryFeedRouteRequest request) {
    final session = request.session;
    final controllers = request.controllers;
    return MaterialPageRoute<void>(
      builder: (context) => BlocProvider(
        create: (_) => controllers.discoveryFeed(request.query)..load(),
        child: DiscoveryFeedScreen(
          request: DiscoveryFeedRequest(
            query: request.query,
            playbackPort: controllers.videoPlaybackPort,
            shareWorkflow: controllers.videoShareWorkflow,
            createComments: controllers.comments,
            onOpenProfile: (profileId) => Navigator.of(context).push(
              AppRouter.profile(
                ProfileRouteRequest(
                  session: session,
                  profileId: profileId,
                  controllers: controllers,
                  onSignedOut: request.onSignedOut,
                  onCurrentProfileUpdated: request.onCurrentProfileUpdated,
                ),
              ),
            ),
            onOpenHashtag: (hashtag) => Navigator.of(context).push(
              AppRouter.discoveryFeed(
                DiscoveryFeedRouteRequest(
                  session: session,
                  query: hashtag,
                  controllers: controllers,
                  onSignedOut: request.onSignedOut,
                  onCurrentProfileUpdated: request.onCurrentProfileUpdated,
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }

  static Route<void> profile(ProfileRouteRequest request) {
    final session = request.session;
    final controllers = request.controllers;
    return MaterialPageRoute<void>(
      builder: (context) {
        final isCurrentUser = session.profile.id == request.profileId;
        final cubit = controllers.profile(
          session.profile,
          request.profileId,
          onCurrentProfileUpdated: isCurrentUser
              ? request.onCurrentProfileUpdated
              : null,
        )..load();
        final editor = _RoutedProfileEditor(
          session,
          controllers,
          cubit,
          request.onCurrentProfileUpdated,
        );
        return BlocProvider(
          create: (_) => cubit,
          child: ProfileScreen(
            onSignedOut: request.onSignedOut,
            onEditProfile: isCurrentUser ? () => editor.open(context) : null,
          ),
        );
      },
    );
  }

  static Route<ProfileSummary> editProfile({
    required UserSession session,
    required AppControllerFactory controllers,
  }) {
    return MaterialPageRoute<ProfileSummary>(
      builder: (_) => BlocProvider(
        create: (_) => controllers.profileEdit(session.identity),
        child: ProfileEditScreen(initial: session.profile),
      ),
    );
  }

  static Route<void> settings(AppControllerFactory controllers) {
    return MaterialPageRoute<void>(
      builder: (context) {
        final updates = AppUpdateScope.maybeOf(context);
        return BlocProvider(
          create: (_) => controllers.settings()..load(),
          child: SettingsScreen(
            appUpdateCubit: updates,
            onCheckForUpdates: updates?.checkNow,
            onOpenWatchHistory: () =>
                Navigator.of(context).push(AppRouter.watchHistory(controllers)),
          ),
        );
      },
    );
  }

  static Route<void> watchHistory(AppControllerFactory controllers) {
    return MaterialPageRoute<void>(
      builder: (_) => BlocProvider(
        create: (_) => controllers.watchHistory()..load(),
        child: const WatchHistoryScreen(),
      ),
    );
  }
}

final class _RoutedProfileEditor {
  const _RoutedProfileEditor(
    this._session,
    this._controllers,
    this._cubit,
    this._onUpdated,
  );

  final UserSession _session;
  final AppControllerFactory _controllers;
  final ProfileCubit _cubit;
  final ValueChanged<ProfileSummary>? _onUpdated;

  Future<void> open(BuildContext context) async {
    final profile = _cubit.state.details?.profile ?? _session.profile;
    final updated = await Navigator.of(context).push(
      AppRouter.editProfile(
        session: _session.withProfile(profile),
        controllers: _controllers,
      ),
    );
    if (!context.mounted || updated == null) return;
    _cubit.updateCurrentUser(updated);
    _onUpdated?.call(updated);
  }
}
