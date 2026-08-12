import 'dart:async';

import 'package:flutter/widgets.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/app/profile_route_request.dart';
import 'package:ghostr/app/router/app_router.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';

final class HomeNavigation {
  const HomeNavigation({
    required this.session,
    required this.controllers,
    required this.onSignedOut,
    required this.pushCovering,
    this.onCurrentProfileUpdated,
  });

  final UserSession session;
  final AppControllerFactory controllers;
  final VoidCallback onSignedOut;
  final Future<void> Function(Route<void> route) pushCovering;
  final ValueChanged<ProfileSummary>? onCurrentProfileUpdated;

  void openProfile(ProfileId profileId) {
    unawaited(
      pushCovering(
        AppRouter.profile(
          ProfileRouteRequest(
            session: session,
            profileId: profileId,
            controllers: controllers,
            onSignedOut: onSignedOut,
            onCurrentProfileUpdated: onCurrentProfileUpdated,
          ),
        ),
      ),
    );
  }

  void openDiscoveryFeed(String query) {
    unawaited(
      pushCovering(
        AppRouter.discoveryFeed(
          DiscoveryFeedRouteRequest(
            session: session,
            query: query,
            controllers: controllers,
            onSignedOut: onSignedOut,
            onCurrentProfileUpdated: onCurrentProfileUpdated,
          ),
        ),
      ),
    );
  }
}
