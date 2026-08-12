import 'package:flutter/widgets.dart';
import 'package:ghostr/app/profile_route_request.dart';
import 'package:ghostr/app/router/app_router.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

/// The routes a session screen chains into — creator profiles, discovery
/// feeds, and profile feeds — inheriting the origin's session plumbing.
final class FeedRouteLinks {
  const FeedRouteLinks(this._context, this._request);

  final BuildContext _context;
  final SessionRouteRequest _request;

  Future<void> openProfile(ProfileId profileId) {
    return _push(
      AppRouter.profile(
        ProfileRouteRequest(
          profileId: profileId,
          session: _request.session,
          controllers: _request.controllers,
          onSignedOut: _request.onSignedOut,
          onCurrentProfileUpdated: _request.onCurrentProfileUpdated,
        ),
      ),
    );
  }

  Future<void> openHashtag(String hashtag) {
    return _push(
      AppRouter.discoveryFeed(
        DiscoveryFeedRouteRequest(
          query: hashtag,
          session: _request.session,
          controllers: _request.controllers,
          onSignedOut: _request.onSignedOut,
          onCurrentProfileUpdated: _request.onCurrentProfileUpdated,
        ),
      ),
    );
  }

  Future<void> openProfileFeed(VideoPost post) {
    return _push(
      AppRouter.profileFeed(
        ProfileFeedRouteRequest(
          post: post,
          session: _request.session,
          controllers: _request.controllers,
          onSignedOut: _request.onSignedOut,
          onCurrentProfileUpdated: _request.onCurrentProfileUpdated,
        ),
      ),
    );
  }

  Future<void> _push(Route<void> route) => Navigator.of(_context).push(route);
}
