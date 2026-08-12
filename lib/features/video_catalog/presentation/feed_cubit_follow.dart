part of 'feed_cubit.dart';

extension FeedCubitFollowActions on FeedCubit {
  Future<void> followCreator(ProfileSummary creator) async {
    if (!_canStartFollow(creator.id)) return;
    final workflow = _dependencies.followProfile!;
    final request = _startFollow(creator.id);
    try {
      await workflow.follow(creator);
      _acceptFollow(creator.id, request);
    } on AppFailure catch (failure) {
      _rejectFollow(request, creator, failure.message);
    } on Object catch (error, stackTrace) {
      _rejectFollow(
        request,
        creator,
        feedFollowFailureMessage(FeedOperationFailure(error, stackTrace)),
      );
    } finally {
      _finishFollow(creator.id, request);
    }
  }

  bool _canStartFollow(ProfileId creatorId) {
    final current = state;
    return current is FeedLoaded &&
        _dependencies.followProfile != null &&
        current.canFollow(creatorId);
  }

  int _startFollow(ProfileId creatorId) {
    final request = (_followRequests[creatorId] ?? 0) + 1;
    _followRequests[creatorId] = request;
    _setFollows(_follows.starting(creatorId));
    return request;
  }

  void _acceptFollow(ProfileId creatorId, int request) {
    if (_acceptsFollow(creatorId, request)) {
      _setFollows(_follows.accepted(creatorId));
    }
  }

  void _finishFollow(ProfileId creatorId, int request) {
    if (_acceptsFollow(creatorId, request)) {
      _followRequests.remove(creatorId);
    }
  }

  Future<void> _reloadFollows() async {
    final request = ++_followLoadRequest;
    final viewerId = _dependencies.viewerId;
    if (!_canReloadFollows) {
      _setFollows(FeedFollowState.unavailable(viewerId: viewerId));
      return;
    }
    final social = _dependencies.social!;
    try {
      final followed = await social.loadFollowedProfiles();
      if (!_acceptsFollowLoad(request)) return;
      _setFollows(_follows.refreshed(followed));
    } on Object catch (error, stackTrace) {
      _reportUpdateError(error, stackTrace);
      if (_acceptsFollowLoad(request)) {
        _setFollows(FeedFollowState.unavailable(viewerId: viewerId));
      }
    }
  }

  bool get _canReloadFollows {
    return _dependencies.social != null &&
        _dependencies.viewerId != null &&
        _dependencies.followProfile != null;
  }

  void _rejectFollow(int request, ProfileSummary creator, String message) {
    if (!_acceptsFollow(creator.id, request)) return;
    _setFollows(_follows.rejected(creator.id));
    _showNotice(message);
  }

  bool _acceptsFollow(ProfileId creatorId, int request) {
    return !isClosed && _followRequests[creatorId] == request;
  }

  bool _acceptsFollowLoad(int request) {
    return !isClosed && request == _followLoadRequest;
  }

  void _setFollows(FeedFollowState follows) {
    _follows = follows;
    final current = state;
    if (current is FeedLoaded) _emitState(current.withFollows(follows));
  }
}
