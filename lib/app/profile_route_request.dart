import 'package:flutter/widgets.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

/// What every route pushed over the home shell carries: whose session it
/// serves, the controller factory, and the callbacks it hands onward.
abstract base class SessionRouteRequest {
  const SessionRouteRequest({
    required this.session,
    required this.controllers,
    required this.onSignedOut,
    this.onCurrentProfileUpdated,
  });

  final UserSession session;
  final AppControllerFactory controllers;
  final VoidCallback onSignedOut;
  final ValueChanged<ProfileSummary>? onCurrentProfileUpdated;
}

final class ProfileRouteRequest extends SessionRouteRequest {
  const ProfileRouteRequest({
    required this.profileId,
    required super.session,
    required super.controllers,
    required super.onSignedOut,
    super.onCurrentProfileUpdated,
  });

  final ProfileId profileId;
}

final class DiscoveryFeedRouteRequest extends SessionRouteRequest {
  const DiscoveryFeedRouteRequest({
    required this.query,
    required super.session,
    required super.controllers,
    required super.onSignedOut,
    super.onCurrentProfileUpdated,
  });

  final String query;
}

final class ProfileFeedRouteRequest extends SessionRouteRequest {
  const ProfileFeedRouteRequest({
    required this.post,
    required super.session,
    required super.controllers,
    required super.onSignedOut,
    super.onCurrentProfileUpdated,
  });

  /// The tapped video: whose profile shelf to play and where to start it.
  final VideoPost post;
}
