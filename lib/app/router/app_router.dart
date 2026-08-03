import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:ghostr/features/settings/presentation/settings_screen.dart';
import 'package:ghostr/features/video_catalog/presentation/discovery_feed_screen.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_screen.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/watch_history/presentation/watch_history_screen.dart';

abstract final class AppRouter {
  /// A swipeable video feed for one search query or `#hashtag`.
  static Route<void> discoveryFeed({
    required UserSession session,
    required String query,
    required AppControllerFactory controllers,
    required VoidCallback onSignedOut,
  }) {
    return MaterialPageRoute<void>(
      builder: (context) => BlocProvider(
        create: (_) => controllers.discoveryFeed(query)..load(),
        child: DiscoveryFeedScreen(
          request: DiscoveryFeedRequest(
            query: query,
            playbackPort: controllers.videoPlaybackPort,
            createComments: controllers.comments,
            onOpenProfile: (profileId) => Navigator.of(context).push(
              AppRouter.profile(
                session: session,
                profileId: profileId,
                controllers: controllers,
                onSignedOut: onSignedOut,
              ),
            ),
            onOpenHashtag: (hashtag) => Navigator.of(context).push(
              AppRouter.discoveryFeed(
                session: session,
                query: hashtag,
                controllers: controllers,
                onSignedOut: onSignedOut,
              ),
            ),
          ),
        ),
      ),
    );
  }

  static Route<void> profile({
    required UserSession session,
    required ProfileId profileId,
    required AppControllerFactory controllers,
    required VoidCallback onSignedOut,
  }) {
    return MaterialPageRoute<void>(
      builder: (_) => BlocProvider(
        create: (_) => controllers.profile(session.profile, profileId)..load(),
        child: ProfileScreen(onSignedOut: onSignedOut),
      ),
    );
  }

  static Route<void> settings(AppControllerFactory controllers) {
    return MaterialPageRoute<void>(
      builder: (context) => BlocProvider(
        create: (_) => controllers.settings()..load(),
        child: SettingsScreen(
          onOpenWatchHistory: () =>
              Navigator.of(context).push(AppRouter.watchHistory(controllers)),
        ),
      ),
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
