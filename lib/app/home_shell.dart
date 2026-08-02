import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/app/router/app_router.dart';
import 'package:ghostr/features/activity/presentation/activity_screen.dart';
import 'package:ghostr/features/compose/presentation/compose_screen.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:ghostr/features/session/presentation/session_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_screen.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_screen.dart';
import 'package:ghostr/features/video_catalog/presentation/search_screen.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

enum HomeTab {
  home('Home', Icons.home_rounded),
  search('Search', Icons.search_rounded),
  create('Create', Icons.add_box_rounded),
  activity('Activity', Icons.notifications_rounded),
  profile('Profile', Icons.person_rounded);

  const HomeTab(this.label, this.icon);

  final String label;
  final IconData icon;
}

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
  int _contentRevision = 0;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: _currentScreen(),
      bottomNavigationBar: BottomNavigationBar(
        currentIndex: HomeTab.values.indexOf(_currentTab),
        type: BottomNavigationBarType.fixed,
        backgroundColor: Theme.of(context).scaffoldBackgroundColor,
        selectedItemColor: Theme.of(context).colorScheme.primary,
        unselectedItemColor: AppPalette.mutedForeground,
        onTap: (index) => setState(() => _currentTab = HomeTab.values[index]),
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

  Widget _currentScreen() {
    return switch (_currentTab) {
      HomeTab.home => _home(),
      HomeTab.search => _search(),
      HomeTab.create => _create(),
      HomeTab.activity => _activity(),
      HomeTab.profile => _profile(),
    };
  }

  Widget _home() => BlocProvider(
        key: ValueKey('feed-$_contentRevision'),
        create: (_) => widget.controllers.feed()..load(),
        child: FeedScreen(
          onOpenProfile: _openProfile,
          playbackPort: widget.controllers.videoPlaybackPort,
          createComments: widget.controllers.comments,
        ),
      );

  Widget _search() => BlocProvider(
        key: ValueKey('search-$_contentRevision'),
        create: (_) => widget.controllers.search(),
        child: SearchScreen(onOpenProfile: _openProfile),
      );

  Widget _create() => BlocProvider(
        create: (_) => widget.controllers.compose()..recoverLostVideo(),
        child: ComposeScreen(
          session: widget.session,
          playbackPort: widget.controllers.videoPlaybackPort,
        ),
      );

  Widget _activity() => BlocProvider(
        create: (_) => widget.controllers.activity()..load(),
        child: const ActivityScreen(),
      );

  Widget _profile() => BlocProvider(
        create: (_) => widget.controllers
            .profile(widget.session.profile, widget.session.profile.id)
          ..load(),
        child: ProfileScreen(
          onOpenSettings: _openSettings,
          onSignedOut: _signOut,
        ),
      );

  void _openProfile(ProfileId profileId) {
    unawaited(_openProfileRoute(profileId));
  }

  Future<void> _openProfileRoute(ProfileId profileId) async {
    await Navigator.of(context).push(AppRouter.profile(
      session: widget.session,
      profileId: profileId,
      controllers: widget.controllers,
      onSignedOut: _signOut,
    ));
    if (mounted) setState(() => _contentRevision += 1);
  }

  void _openSettings() {
    Navigator.of(context).push(AppRouter.settings(widget.controllers));
  }

  void _signOut() {
    context.read<SessionCubit>().signOut();
  }
}
