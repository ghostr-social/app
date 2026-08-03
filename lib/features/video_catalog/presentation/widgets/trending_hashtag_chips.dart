import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/video_catalog/presentation/trending_hashtags_cubit.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

/// Tappable trending tags shown before the viewer has typed anything.
class TrendingHashtagChips extends StatelessWidget {
  const TrendingHashtagChips({required this.onOpenHashtag, super.key});

  final ValueChanged<String> onOpenHashtag;

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<TrendingHashtagsCubit, TrendingHashtagsState>(
      builder: (context, state) => switch (state) {
        TrendingHashtagsReady(tags: final tags) => _chips(context, tags),
        TrendingHashtagsLoading() ||
        TrendingHashtagsUnavailable() =>
          const SizedBox.shrink(),
      },
    );
  }

  Widget _chips(BuildContext context, List<String> tags) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('Trending now', style: Theme.of(context).textTheme.titleMedium),
        const SizedBox(height: AppSpacing.sm),
        Wrap(
          spacing: AppSpacing.sm,
          runSpacing: AppSpacing.sm,
          children: [
            for (final tag in tags)
              ActionChip(
                label: Text('#$tag'),
                onPressed: () => onOpenHashtag('#$tag'),
              ),
          ],
        ),
      ],
    );
  }
}
