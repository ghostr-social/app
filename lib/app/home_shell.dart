import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/app/home_tab.dart';
import 'package:ghostr/app/home_tab_bar.dart';
import 'package:ghostr/app/home_tab_stack.dart';
import 'package:ghostr/app/home_navigation.dart';
import 'package:ghostr/app/profile_edit_navigation.dart';
import 'package:ghostr/app/router/app_router.dart';
import 'package:ghostr/core/media/incoming_video_share.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/features/activity/presentation/activity_cubit.dart';
import 'package:ghostr/features/activity/presentation/activity_screen.dart';
import 'package:ghostr/features/compose/presentation/compose_cubit.dart';
import 'package:ghostr/features/compose/presentation/compose_screen.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:ghostr/features/session/presentation/session_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_screen.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_screen.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_cubit.dart';
import 'package:ghostr/app/search_tab.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';

part 'home_shell_incoming_video.dart';
part 'home_shell_navigation.dart';

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
  late final ComposeCubit _composeCubit;
  late final StreamSubscription<IncomingVideoShareEvent> _incomingVideoShares;
  Future<void>? _composeRecovery;
  bool _composeRecoveryStarted = false;
  int _latestIncomingVideoRequest = 0;
  final Map<int, SelectedMedia> _pendingIncomingVideos = {};
  final Map<String, SelectedMedia> _unmountedIncomingVideos = {};

  @override
  void initState() {
    super.initState();
    _initializeIncomingVideos();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: HomeTabStack(
        currentTab: _currentTab,
        visitedTabs: _visitedTabs,
        tabBuilder: _tabScreen,
      ),
      bottomNavigationBar: HomeTabBar(
        currentTab: _currentTab,
        onSelect: _selectTab,
      ),
    );
  }

  Widget _tabScreen(HomeTab tab) {
    return switch (tab) {
      HomeTab.home => _home(),
      HomeTab.search => _search(),
      HomeTab.create => _create(),
      HomeTab.activity => _activity(),
      HomeTab.profile => _profile(),
    };
  }

  void _selectTab(int index) {
    final selected = HomeTab.values[index];
    final isReselectedHome =
        selected == HomeTab.home && selected == _currentTab;
    final shouldRefresh = _visitedTabs.contains(selected);
    _activateTab(selected);
    if (isReselectedHome) {
      if (_feedCubit?.reload() case final reload?) unawaited(reload);
    } else if (shouldRefresh) {
      _refreshTab(selected);
    }
  }

  void _activateTab(HomeTab selected) {
    setState(() {
      _currentTab = selected;
      _visitedTabs.add(selected);
    });
  }

  void _setRouteCovered(bool isCovered) {
    setState(() => _isRouteCovered = isCovered);
    if (!isCovered) _refreshTab(_currentTab);
  }

  Widget _home() => BlocProvider(
    create: (_) => _createFeedCubit(),
    child: FeedScreen(
      bindings: FeedScreenBindings(
        onOpenProfile: _openProfile,
        onOpenHashtag: _openHashtag,
        playbackPort: widget.controllers.videoPlaybackPort,
        shareWorkflow: widget.controllers.videoShareWorkflow,
        createComments: widget.controllers.comments,
        isActive: _currentTab == HomeTab.home && !_isRouteCovered,
      ),
    ),
  );
  Widget _search() => SearchTab(
    createSearchCubit: _createSearchCubit,
    createTrendingCubit: () => widget.controllers.trending()..load(),
    onOpenProfile: _openProfile,
    onOpenFeed: _openDiscoveryFeed,
  );
  Widget _create() {
    _recoverComposeDraft();
    return BlocProvider.value(
      value: _composeCubit,
      child: ComposeScreen(
        session: widget.session,
        playbackPort: widget.controllers.videoPlaybackPort,
        isActive: _currentTab == HomeTab.create && !_isRouteCovered,
        onPreviewMounted: _claimIncomingVideoPreview,
      ),
    );
  }

  Widget _activity() => BlocProvider(
    create: (_) => _createActivityCubit(),
    child: const ActivityScreen(),
  );
  Widget _profile() => BlocProvider(
    create: (_) => _createProfileCubit(),
    child: ProfileScreen(
      onOpenSettings: _openSettings,
      onEditProfile: _editProfile,
      onSignedOut: _signOut,
      onOpenVideo: _openProfileVideo,
    ),
  );
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

  @override
  void dispose() {
    _disposeIncomingVideos();
    super.dispose();
  }
}
