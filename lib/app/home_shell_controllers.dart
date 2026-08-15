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
    switch (tab) {
      case HomeTab.home:
        return _runRefresh(_feedCubit?.refresh());
      case HomeTab.search:
        return _runRefresh(_searchCubit?.refresh());
      case HomeTab.activity:
        return _runRefresh(_activityCubit?.load());
      case HomeTab.profile:
        return _runRefresh(_profileCubit?.load());
      case HomeTab.create:
        return;
    }
  }

  void _deactivateSearchWhenLeaving(HomeTab selected) {
    if (selected == _currentTab) return;
    if (_currentTab == HomeTab.search) _searchCubit?.deactivate();
  }

  void _refreshActivatedTab(
    HomeTab selected,
    bool isReselectedHome,
    bool shouldRefresh,
  ) {
    if (isReselectedHome) return _runRefresh(_feedCubit?.reload());
    if (shouldRefresh) _refreshTab(selected);
  }

  void _runRefresh(Future<void>? refresh) {
    if (refresh != null) unawaited(refresh);
  }
}
