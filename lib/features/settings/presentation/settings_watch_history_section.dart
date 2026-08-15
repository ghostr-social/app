import 'package:flutter/material.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

class SettingsWatchHistorySection extends StatelessWidget {
  const SettingsWatchHistorySection({
    required this.onOpenWatchHistory,
    super.key,
  });

  final VoidCallback? onOpenWatchHistory;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Text('Watch history', style: Theme.of(context).textTheme.titleLarge),
        const SizedBox(height: AppSpacing.xs),
        const Text(
          'Watched videos stay out of For You and search until you clear '
          'your watch history.',
          key: Key('watched-video-policy'),
        ),
        const SizedBox(height: AppSpacing.sm),
        ListTile(
          key: const Key('watch-history-entry'),
          title: const Text('View watch history'),
          trailing: const Icon(Icons.chevron_right),
          onTap: onOpenWatchHistory,
        ),
      ],
    );
  }
}
