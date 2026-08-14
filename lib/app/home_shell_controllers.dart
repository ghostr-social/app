part of 'home_shell.dart';

extension _HomeShellControllers on _HomeShellState {
  FeedCubit _createFeedCubit() {
    return _feedCubit = widget.controllers.feed(
      viewerId: widget.session.profile.id,
    )..load();
  }

  SearchCubit _createSearchCubit() {
    return _searchCubit ??= widget.controllers.search();
  }

  ActivityCubit _createActivityCubit() {
    return _activityCubit = widget.controllers.activity()..load();
  }

  ProfileCubit _createProfileCubit() {
    return _profileCubit = widget.controllers.profile(
      widget.session.profile,
      widget.session.profile.id,
      onCurrentProfileUpdated: context.read<SessionCubit>().updateProfile,
    )..load();
  }

  void _refreshTab(HomeTab tab) {
    final refresh = switch (tab) {
      HomeTab.home => _feedCubit?.refresh(),
      HomeTab.activity => _activityCubit?.load(),
      HomeTab.profile => _profileCubit?.load(),
      HomeTab.search || HomeTab.create => null,
    };
    if (refresh != null) unawaited(refresh);
  }
}
