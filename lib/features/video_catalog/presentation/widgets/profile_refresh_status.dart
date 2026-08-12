import 'package:flutter/material.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

class ProfileRefreshStatus extends StatelessWidget {
  const ProfileRefreshStatus({
    required this.isRefreshing,
    required this.error,
    required this.onRetry,
    super.key,
  });

  final bool isRefreshing;
  final String? error;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    if (isRefreshing) return const _RefreshingProfile();
    final message = error;
    if (message == null) return const SizedBox.shrink();
    return _RefreshFailure(message: message, onRetry: onRetry);
  }
}

class _RefreshingProfile extends StatelessWidget {
  const _RefreshingProfile();

  @override
  Widget build(BuildContext context) {
    return Semantics(
      liveRegion: true,
      label: 'Refreshing profile',
      child: ExcludeSemantics(
        child: Column(
          children: [
            const LinearProgressIndicator(),
            Padding(
              padding: const EdgeInsets.all(AppSpacing.xs),
              child: Text(
                'Refreshing profile',
                style: Theme.of(context).textTheme.labelMedium,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _RefreshFailure extends StatelessWidget {
  const _RefreshFailure({required this.message, required this.onRetry});

  final String message;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    return ColoredBox(
      color: Theme.of(context).colorScheme.errorContainer,
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: AppSpacing.md),
        child: Row(
          children: [
            Expanded(child: Text(message)),
            TextButton(onPressed: onRetry, child: const Text('Retry')),
          ],
        ),
      ),
    );
  }
}
