import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/comments/presentation/comments_sheet.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_screen_bindings.dart';
import 'package:ghostr/features/video_catalog/presentation/video_share_feed_scope.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_kind_selector.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_page_view.dart';
import 'package:ghostr/features/video_sharing/presentation/video_share_cubit.dart';
import 'package:ghostr/shared/widgets/async_state_panel.dart';
import 'package:ghostr/shared/widgets/loading_panel.dart';

export 'feed_screen_bindings.dart';

part 'feed_screen_actions.dart';

class FeedScreen extends StatefulWidget {
  const FeedScreen({required this.bindings, super.key});

  final FeedScreenBindings bindings;

  @override
  State<FeedScreen> createState() => _FeedScreenState();
}

class _FeedScreenState extends State<FeedScreen> with WidgetsBindingObserver {
  bool _commentsOpen = false;
  bool _appIsResumed = true;
  FeedCubit? _cubit;

  @override
  void initState() {
    super.initState();
    final binding = WidgetsBinding.instance;
    binding.addObserver(this);
    final lifecycle = binding.lifecycleState;
    _appIsResumed = lifecycle == null || lifecycle == AppLifecycleState.resumed;
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final cubit = context.read<FeedCubit>();
    if (identical(_cubit, cubit)) return;
    _cubit?.surfaceVisibilityChanged(false);
    _cubit = cubit;
    cubit.surfaceVisibilityChanged(_isVisible);
  }

  @override
  void didUpdateWidget(covariant FeedScreen oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.bindings.isActive == widget.bindings.isActive) return;
    _cubit?.surfaceVisibilityChanged(_isVisible);
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    final resumed = state == AppLifecycleState.resumed;
    if (_appIsResumed == resumed) return;
    _appIsResumed = resumed;
    _cubit?.surfaceVisibilityChanged(_isVisible);
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    _cubit?.surfaceVisibilityChanged(false);
    super.dispose();
  }

  bool get _isVisible {
    return widget.bindings.isActive && _appIsResumed && !_commentsOpen;
  }

  @override
  Widget build(BuildContext context) {
    return VideoShareFeedScope(
      workflow: widget.bindings.shareWorkflow,
      child: BlocListener<FeedCubit, FeedState>(
        listenWhen: _hasNewNotice,
        listener: _showNotice,
        child: BlocBuilder<FeedCubit, FeedState>(builder: _buildFeed),
      ),
    );
  }

  bool _hasNewNotice(FeedState previous, FeedState current) {
    return current is FeedLoaded && current.notice != null;
  }

  void _showNotice(BuildContext context, FeedState state) {
    final message = (state as FeedLoaded).notice!;
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(SnackBar(content: Text(message)));
    context.read<FeedCubit>().clearNotice();
  }

  Widget _buildFeed(BuildContext context, FeedState state) {
    return FeedKindOverlay(
      selected: state.kind,
      visible: widget.bindings.showFeedKindSelector,
      onSelected: (kind) => unawaited(context.read<FeedCubit>().load(kind)),
      child: _feedContent(context, state),
    );
  }

  Widget _feedContent(BuildContext context, FeedState state) {
    if (_hidesLoading(state)) return const SizedBox.shrink();
    return _visibleFeedContent(context, state);
  }

  bool _hidesLoading(FeedState state) {
    return !_isVisible && state is FeedLoading;
  }

  Widget _visibleFeedContent(BuildContext context, FeedState state) {
    return switch (state) {
      FeedLoading() => const LoadingPanel(label: 'Loading video feed'),
      FeedEmpty() => _emptyFeed(context),
      FeedFailure(message: final message) => _feedError(context, message),
      FeedLoaded() => _feedPages(context, state),
    };
  }

  Widget _feedError(BuildContext context, String message) {
    return AsyncStatePanel(
      icon: Icons.wifi_tethering_error,
      title: 'Feed unavailable',
      message: message,
      actionLabel: 'Retry',
      onAction: context.read<FeedCubit>().retry,
    );
  }

  // The feed hunts on its own while this panel is visible; the action just
  // lets an impatient viewer skip the backoff delay.
  Widget _emptyFeed(BuildContext context) {
    return AsyncStatePanel(
      icon: Icons.travel_explore,
      title: 'Hunting for videos',
      message:
          'The search keeps running — new clips appear the moment a '
          'relay hands them over. Following creators fills this feed faster.',
      actionLabel: 'Search again',
      onAction: context.read<FeedCubit>().retry,
    );
  }

  Widget _feedPages(BuildContext context, FeedLoaded state) {
    return FeedPageView(
      key: ValueKey(state.posts.first.id.value),
      itemCount: state.posts.length,
      initialPage: state.activeIndex,
      onPageChanged: context.read<FeedCubit>().pageChanged,
      itemBuilder: (_, index) => index == state.activeIndex
          ? _feedCard(context, state, index)
          : const ColoredBox(color: Colors.black),
    );
  }

  Widget _feedCard(BuildContext context, FeedLoaded state, int index) {
    final post = state.posts[index];
    return BlocBuilder<VideoShareCubit, VideoShareState>(
      builder: (context, sharing) => FeedCard(
        key: ValueKey(post.id.value),
        post: post,
        playback: FeedCardPlayback(
          port: widget.bindings.playbackPort,
          isActive: _isVisible && index == state.activeIndex,
        ),
        actions: _actions(context, state, post, sharing),
      ),
    );
  }

  void _setCommentsOpen(bool value) {
    setState(() => _commentsOpen = value);
    _cubit?.surfaceVisibilityChanged(_isVisible);
  }
}
