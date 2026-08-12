import 'package:flutter/widgets.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';

final class ProfileRouteRequest {
  const ProfileRouteRequest({
    required this.session,
    required this.profileId,
    required this.controllers,
    required this.onSignedOut,
    this.onCurrentProfileUpdated,
  });

  final UserSession session;
  final ProfileId profileId;
  final AppControllerFactory controllers;
  final VoidCallback onSignedOut;
  final ValueChanged<ProfileSummary>? onCurrentProfileUpdated;
}

final class DiscoveryFeedRouteRequest {
  const DiscoveryFeedRouteRequest({
    required this.session,
    required this.query,
    required this.controllers,
    required this.onSignedOut,
    this.onCurrentProfileUpdated,
  });

  final UserSession session;
  final String query;
  final AppControllerFactory controllers;
  final VoidCallback onSignedOut;
  final ValueChanged<ProfileSummary>? onCurrentProfileUpdated;
}
