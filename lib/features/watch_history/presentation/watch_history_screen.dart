import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/presentation/watch_history_cubit.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';
import 'package:ghostr/shared/widgets/async_state_panel.dart';
import 'package:ghostr/shared/widgets/loading_panel.dart';

class WatchHistoryScreen extends StatelessWidget {
  const WatchHistoryScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Watch history'),
        actions: [
          BlocBuilder<WatchHistoryCubit, WatchHistoryState>(
            builder: _clearAction,
          ),
        ],
      ),
      body: BlocBuilder<WatchHistoryCubit, WatchHistoryState>(
        builder: _buildHistory,
      ),
    );
  }

  Widget _clearAction(BuildContext context, WatchHistoryState state) {
    if (state is! WatchHistoryLoaded && state is! WatchHistoryFailure) {
      return const SizedBox.shrink();
    }
    return IconButton(
      tooltip: 'Clear watch history',
      onPressed: context.read<WatchHistoryCubit>().clear,
      icon: const Icon(Icons.delete_outline),
    );
  }

  Widget _buildHistory(BuildContext context, WatchHistoryState state) {
    return switch (state) {
      WatchHistoryLoading() => const LoadingPanel(
        label: 'Loading watch history',
      ),
      WatchHistoryEmpty() => _emptyHistory(),
      WatchHistoryLoaded(entries: final entries) => _historyList(
        context,
        entries,
      ),
      WatchHistoryFailure(message: final message) => _errorHistory(
        context,
        message,
      ),
    };
  }

  Widget _emptyHistory() {
    return const AsyncStatePanel(
      icon: Icons.history,
      title: 'No watched videos yet',
      message:
          'Videos you watch in the feed are remembered here so the '
          'feed can skip them next time.',
    );
  }

  Widget _errorHistory(BuildContext context, String message) {
    return AsyncStatePanel(
      icon: Icons.error_outline,
      title: 'Watch history unavailable',
      message: message,
      actionLabel: 'Retry',
      onAction: context.read<WatchHistoryCubit>().load,
    );
  }

  Widget _historyList(BuildContext context, List<WatchHistoryEntry> entries) {
    return ListView.separated(
      padding: const EdgeInsets.all(AppSpacing.lg),
      itemCount: entries.length,
      separatorBuilder: (_, __) => const SizedBox(height: AppSpacing.sm),
      itemBuilder: (_, index) => _historyTile(context, entries[index]),
    );
  }

  Widget _historyTile(BuildContext context, WatchHistoryEntry entry) {
    return ListTile(
      tileColor: Theme.of(context).colorScheme.surface,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(AppRadius.control),
      ),
      title: Text(entry.title, maxLines: 2, overflow: TextOverflow.ellipsis),
      subtitle: Text(
        '${entry.creatorName} • ${_watchedLabel(entry.watchedAt)}',
      ),
    );
  }

  String _watchedLabel(DateTime watchedAt) {
    final local = watchedAt.toLocal();
    final date = '${local.year}-${_pad(local.month)}-${_pad(local.day)}';
    return 'Watched $date ${_pad(local.hour)}:${_pad(local.minute)}';
  }

  String _pad(int value) => value.toString().padLeft(2, '0');
}
