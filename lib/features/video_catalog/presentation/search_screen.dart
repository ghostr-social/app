import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/search_results_view.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/trending_hashtag_chips.dart';
import 'package:ghostr/shared/widgets/async_state_panel.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';
import 'package:ghostr/shared/widgets/loading_panel.dart';

class SearchScreen extends StatefulWidget {
  const SearchScreen({
    required this.onOpenProfile,
    required this.onOpenFeed,
    super.key,
  });

  final ValueChanged<ProfileId> onOpenProfile;

  /// Opens a query or `#hashtag` as a full swipeable video feed.
  final ValueChanged<String> onOpenFeed;

  @override
  State<SearchScreen> createState() => _SearchScreenState();
}

class _SearchScreenState extends State<SearchScreen> {
  final _controller = TextEditingController();

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final query = context.read<SearchCubit>().state.query;
    if (query.isNotEmpty && _controller.text.trim() != query) {
      _controller.text = query;
    }
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: Padding(
        padding: const EdgeInsets.all(AppSpacing.lg),
        child: Column(
          children: [
            _searchBar(),
            const SizedBox(height: AppSpacing.lg),
            Expanded(
              child: BlocConsumer<SearchCubit, SearchState>(
                listenWhen: _hasSideEffect,
                listener: _handleSideEffect,
                builder: _buildContent,
              ),
            ),
          ],
        ),
      ),
    );
  }

  bool _hasSideEffect(SearchState previous, SearchState current) {
    return _hasExternalQuery(current) ||
        (current.notice != null && current.notice != previous.notice);
  }

  bool _hasExternalQuery(SearchState state) {
    return state.query.isNotEmpty && state.query != _controller.text.trim();
  }

  void _handleSideEffect(BuildContext context, SearchState state) {
    if (_hasExternalQuery(state)) _controller.text = state.query;
    final notice = state.notice;
    if (notice == null) return;
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(notice)),
    );
    context.read<SearchCubit>().clearNotice();
  }

  Widget _searchBar() {
    return Row(
      children: [
        Expanded(
          child: TextField(
            controller: _controller,
            decoration: const InputDecoration(
              hintText: 'Search videos, creators, or #hashtags',
            ),
            onChanged: context.read<SearchCubit>().queryChanged,
            onSubmitted: _search,
          ),
        ),
        const SizedBox(width: AppSpacing.sm),
        FilledButton(
            onPressed: () => _search(_controller.text),
            child: const Text('Search')),
      ],
    );
  }

  Widget _buildContent(BuildContext context, SearchState state) {
    return switch (state) {
      SearchIdle() => _discoverPanel(),
      SearchLoading() => const LoadingPanel(label: 'Searching Nostr'),
      SearchEmpty(query: final query) => _emptyResults(query),
      SearchLoaded() => _results(context, state),
      SearchFailure(message: final message) => _errorState(message),
    };
  }

  Widget _results(BuildContext context, SearchLoaded state) {
    return SearchResultsView(
      results: state,
      actions: SearchResultsActions(
        onOpenProfile: widget.onOpenProfile,
        onOpenFeed: () => widget.onOpenFeed(state.query),
        onLoadMore: () => unawaited(context.read<SearchCubit>().loadMore()),
      ),
    );
  }

  Widget _discoverPanel() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        TrendingHashtagChips(onOpenHashtag: widget.onOpenFeed),
        const Expanded(
          child: AsyncStatePanel(
            icon: Icons.search,
            title: 'Search creators and videos',
            message:
                'Type to search all of Nostr — or jump into a trending tag.',
          ),
        ),
      ],
    );
  }

  // The full feed can deliberately dig deeper than this compact result view.
  Widget _emptyResults(String query) {
    return Semantics(
      container: true,
      explicitChildNodes: true,
      label: 'Searching Nostr for more matches',
      child: AsyncStatePanel(
        icon: Icons.manage_search,
        title: 'No matches yet',
        message: 'No relay has returned a playable match yet. Try a creator '
            'handle, caption keyword, or #hashtag.',
        actionLabel: 'Open in feed',
        onAction: () => widget.onOpenFeed(query),
      ),
    );
  }

  Widget _errorState(String message) {
    return AsyncStatePanel(
      icon: Icons.search_off,
      title: 'Search unavailable',
      message: message,
      actionLabel: 'Retry',
      onAction: context.read<SearchCubit>().retry,
    );
  }

  void _search(String query) {
    unawaited(context.read<SearchCubit>().search(query));
  }
}
