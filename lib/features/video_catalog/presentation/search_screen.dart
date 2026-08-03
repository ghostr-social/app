import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/presentation/search_cubit.dart';
import 'package:ghostr/shared/widgets/async_state_panel.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';
import 'package:ghostr/shared/widgets/loading_panel.dart';

class SearchScreen extends StatefulWidget {
  const SearchScreen({required this.onOpenProfile, super.key});

  final ValueChanged<ProfileId> onOpenProfile;

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
                listenWhen: _hasExternalQuery,
                listener: _syncQueryField,
                builder: _buildContent,
              ),
            ),
          ],
        ),
      ),
    );
  }

  bool _hasExternalQuery(SearchState previous, SearchState current) {
    return current.query.isNotEmpty && current.query != _controller.text.trim();
  }

  void _syncQueryField(BuildContext context, SearchState state) {
    _controller.text = state.query;
  }

  Widget _searchBar() {
    return Row(
      children: [
        Expanded(
          child: TextField(
            controller: _controller,
            decoration: const InputDecoration(
              hintText: 'Search creators, clips, or #hashtags',
            ),
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
      SearchIdle() => _initialState(),
      SearchLoading() => const LoadingPanel(label: 'Searching Nostr'),
      SearchEmpty() => _emptyResults(),
      SearchLoaded(results: final results) => _resultList(results),
      SearchFailure(message: final message) => _errorState(message),
    };
  }

  Widget _initialState() {
    return const AsyncStatePanel(
      icon: Icons.search,
      title: 'Search creators and videos',
      message: 'Type a name, handle, or caption to explore the Nostr timeline.',
    );
  }

  Widget _emptyResults() {
    return const AsyncStatePanel(
      icon: Icons.manage_search,
      title: 'No matches found',
      message: 'Try a creator handle, a caption keyword, or a song name.',
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

  Widget _resultList(List<VideoPost> results) {
    return ListView.separated(
      itemCount: results.length,
      separatorBuilder: (_, __) => const SizedBox(height: AppSpacing.sm),
      itemBuilder: (_, index) => _resultTile(results[index]),
    );
  }

  Widget _resultTile(VideoPost post) {
    return ListTile(
      tileColor: Theme.of(context).colorScheme.surface,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(AppRadius.control),
      ),
      title: Text(post.creator.displayName),
      subtitle: Text(post.caption),
      trailing: const Icon(Icons.chevron_right),
      onTap: () => widget.onOpenProfile(post.creator.id),
    );
  }

  void _search(String query) => context.read<SearchCubit>().search(query);
}
