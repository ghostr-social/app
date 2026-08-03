import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/app/home_tab.dart';
import 'package:ghostr/app/router/app_router.dart';
import 'package:ghostr/features/activity/presentation/activity_cubit.dart';
import 'package:ghostr/features/activity/presentation/activity_screen.dart';
import 'package:ghostr/features/compose/presentation/compose_screen.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:ghostr/features/session/presentation/session_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_screen.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_screen.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/search_screen.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

class HomeShell extends StatefulWidget {
  const HomeShell({
    required this.session,
    required this.controllers,
    super.key,
  });

  final UserSession session;
  final AppControllerFactory controllers;

  @override
  State<HomeShell> createState() => _HomeShellState();
}

class _HomeShellState extends State<HomeShell> {
  HomeTab _currentTab = HomeTab.home;
  final Set<HomeTab> _visitedTabs = {HomeTab.home};
  bool _isRouteCovered = false;
  FeedCubit? _feedCubit;
  SearchCubit? _searchCubit;
  ActivityCubit? _activityCubit;
  ProfileCubit? _profileCubit;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: _tabStack(),
      bottomNavigationBar: BottomNavigationBar(
        currentIndex: HomeTab.values.indexOf(_currentTab),
        type: BottomNavigationBarType.fixed,
        backgroundColor: Theme.of(context).scaffoldBackgroundColor,
        selectedItemColor: Theme.of(context).colorScheme.primary,
        unselectedItemColor: AppPalette.mutedForeground,
        onTap: _selectTab,
        items: HomeTab.values
            .map(
              (tab) => BottomNavigationBarItem(
                icon: Icon(tab.icon),
                label: tab.label,
              ),
            )
            .toList(),
      ),
    );
  }

  Widget _tabStack() {
    return IndexedStack(
      index: HomeTab.values.indexOf(_currentTab),
      children: HomeTab.values.map(_tabScreen).toList(),
    );
  }

  Widget _tabScreen(HomeTab tab) {
    if (!_visitedTabs.contains(tab)) return const SizedBox.shrink();
    return KeyedSubtree(
      key: ValueKey('home-tab-${tab.name}'),
      child: switch (tab) {
        HomeTab.home => _home(),
        HomeTab.search => _search(),
        HomeTab.create => _create(),
        HomeTab.activity => _activity(),
        HomeTab.profile => _profile(),
      },
    );
  }

  void _selectTab(int index) {
    final selected = HomeTab.values[index];
    final isReselectedHome =
        selected == HomeTab.home && selected == _currentTab;
    final shouldRefresh = _visitedTabs.contains(selected);
    setState(() {
      _currentTab = selected;
      _visitedTabs.add(selected);
    });
    if (isReselectedHome) {
      final reload = _feedCubit?.reload();
      if (reload != null) unawaited(reload);
    } else if (shouldRefresh) {
      _refreshTab(selected);
    }
  }

  Widget _home() => BlocProvider(
        create: (_) => _createFeedCubit(),
        child: FeedScreen(
          bindings: FeedScreenBindings(
            onOpenProfile: _openProfile,
            onOpenHashtag: _openHashtag,
            playbackPort: widget.controllers.videoPlaybackPort,
            createComments: widget.controllers.comments,
            isActive: _currentTab == HomeTab.home && !_isRouteCovered,
          ),
        ),
      );

  Widget _search() => BlocProvider(
        create: (_) => _createSearchCubit(),
        child: SearchScreen(onOpenProfile: _openProfile),
      );

  Widget _create() => BlocProvider(
        create: (_) => widget.controllers.compose()..recoverLostVideo(),
        child: ComposeScreen(
          session: widget.session,
          playbackPort: widget.controllers.videoPlaybackPort,
          isActive: _currentTab == HomeTab.create,
        ),
      );

  Widget _activity() => BlocProvider(
        create: (_) => _createActivityCubit(),
        child: const ActivityScreen(),
      );

  Widget _profile() => BlocProvider(
        create: (_) => _createProfileCubit(),
        child: ProfileScreen(
          onOpenSettings: _openSettings,
          onSignedOut: _signOut,
        ),
      );

  void _openProfile(ProfileId profileId) {
    unawaited(_openProfileRoute(profileId));
  }

  void _openHashtag(String hashtag) {
    final search = _createSearchCubit();
    setState(() {
      _currentTab = HomeTab.search;
      _visitedTabs.add(HomeTab.search);
    });
    unawaited(search.search(hashtag));
  }

  Future<void> _openProfileRoute(ProfileId profileId) async {
    if (_isRouteCovered) return;
    setState(() => _isRouteCovered = true);
    try {
      await Navigator.of(context).push(AppRouter.profile(
        session: widget.session,
        profileId: profileId,
        controllers: widget.controllers,
        onSignedOut: _signOut,
      ));
    } finally {
      if (mounted) {
        setState(() => _isRouteCovered = false);
        _refreshTab(_currentTab);
      }
    }
  }

  void _openSettings() {
    Navigator.of(context).push(AppRouter.settings(widget.controllers));
  }

  void _signOut() {
    final session = context.read<SessionCubit>();
    Navigator.of(context).popUntil((route) => route.isFirst);
    session.signOut();
  }

  FeedCubit _createFeedCubit() {
    return _feedCubit = widget.controllers.feed()..load();
  }

  SearchCubit _createSearchCubit() {
    return _searchCubit ??= widget.controllers.search();
  }

  ActivityCubit _createActivityCubit() {
    return _activityCubit = widget.controllers.activity()..load();
  }

  ProfileCubit _createProfileCubit() {
    return _profileCubit = widget.controllers
        .profile(widget.session.profile, widget.session.profile.id)
      ..load();
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
