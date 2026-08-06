import 'dart:async';

import 'package:flutter/widgets.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/app/router/app_router.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

final class HomeNavigation {
  const HomeNavigation({
    required this.session,
    required this.controllers,
    required this.onSignedOut,
    required this.pushCovering,
  });

  final UserSession session;
  final AppControllerFactory controllers;
  final VoidCallback onSignedOut;
  final Future<void> Function(Route<void> route) pushCovering;

  void openProfile(ProfileId profileId) {
    unawaited(
      pushCovering(
        AppRouter.profile(
          session: session,
          profileId: profileId,
          controllers: controllers,
          onSignedOut: onSignedOut,
        ),
      ),
    );
  }

  void openDiscoveryFeed(String query) {
    unawaited(
      pushCovering(
        AppRouter.discoveryFeed(
          session: session,
          query: query,
          controllers: controllers,
          onSignedOut: onSignedOut,
        ),
      ),
    );
  }
}
