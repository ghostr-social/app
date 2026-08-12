part of 'home_shell.dart';

extension _HomeShellNavigation on _HomeShellState {
  void _openProfile(ProfileId profileId) => _navigation.openProfile(profileId);

  void _openHashtag(String hashtag) => _openDiscoveryFeed(hashtag);

  void _openProfileVideo(VideoPost post) => _navigation.openProfileFeed(post);

  void _openDiscoveryFeed(String query) => _navigation.openDiscoveryFeed(query);

  HomeNavigation get _navigation => HomeNavigation(
    session: widget.session,
    controllers: widget.controllers,
    onSignedOut: _signOut,
    pushCovering: _pushCovering,
    onCurrentProfileUpdated: context.read<SessionCubit>().updateProfile,
  );

  Future<void> _pushCovering(Route<void> route) async {
    if (_isRouteCovered) return;
    _setRouteCovered(true);
    try {
      await Navigator.of(context).push(route);
    } finally {
      if (mounted) _setRouteCovered(false);
    }
  }

  void _openSettings() {
    Navigator.of(context).push(AppRouter.settings(widget.controllers));
  }

  Future<void> _editProfile() async {
    await openProfileEditor(context, widget.session, widget.controllers, (
      updated,
    ) {
      _profileCubit?.updateCurrentUser(updated);
      context.read<SessionCubit>().updateProfile(updated);
    });
  }

  void _dismissCoveredRoutes() {
    Navigator.of(context).popUntil((route) => route.isFirst);
  }

  void _signOut() {
    final session = context.read<SessionCubit>();
    _dismissCoveredRoutes();
    session.signOut();
  }
}
