import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/activity/domain/activity_item.dart';
import 'package:ghostr/features/activity/presentation/activity_cubit.dart';
import 'package:ghostr/shared/widgets/async_state_panel.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';
import 'package:ghostr/shared/widgets/loading_panel.dart';

class ActivityScreen extends StatelessWidget {
  const ActivityScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: BlocBuilder<ActivityCubit, ActivityState>(
        builder: _buildActivity,
      ),
    );
  }

  Widget _buildActivity(BuildContext context, ActivityState state) {
    return switch (state) {
      ActivityLoading() => const LoadingPanel(label: 'Loading activity'),
      ActivityEmpty() => _emptyActivity(),
      ActivityLoaded(items: final items) => _activityList(context, items),
      ActivityFailure(message: final message) => _errorActivity(
          context,
          message,
        ),
    };
  }

  Widget _emptyActivity() {
    return const AsyncStatePanel(
      icon: Icons.notifications_none,
      title: 'No activity yet',
      message: 'Nostr likes, comments, follows, and publishes appear here.',
    );
  }

  Widget _errorActivity(BuildContext context, String message) {
    return AsyncStatePanel(
      icon: Icons.error_outline,
      title: 'Activity unavailable',
      message: message,
      actionLabel: 'Retry',
      onAction: context.read<ActivityCubit>().load,
    );
  }

  Widget _activityList(BuildContext context, List<ActivityItem> items) {
    return ListView.separated(
      padding: const EdgeInsets.all(AppSpacing.lg),
      itemCount: items.length,
      separatorBuilder: (_, __) => const SizedBox(height: AppSpacing.sm),
      itemBuilder: (_, index) => _activityTile(context, items[index]),
    );
  }

  Widget _activityTile(BuildContext context, ActivityItem item) {
    return ListTile(
      tileColor: Theme.of(context).colorScheme.surface,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(AppRadius.control),
      ),
      title: Text(item.title),
      subtitle: Text(item.body),
    );
  }
}
