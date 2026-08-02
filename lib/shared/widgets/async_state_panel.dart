import 'package:flutter/material.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

class AsyncStatePanel extends StatelessWidget {
  const AsyncStatePanel({
    required this.icon,
    required this.title,
    required this.message,
    this.actionLabel,
    this.onAction,
    super.key,
  });

  final IconData icon;
  final String title;
  final String message;
  final String? actionLabel;
  final VoidCallback? onAction;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(AppSpacing.xl),
        child: _content(context),
      ),
    );
  }

  Widget _content(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        Icon(
          icon,
          size: AppSize.stateIcon,
          color: AppPalette.mutedForeground,
        ),
        const SizedBox(height: AppSpacing.md),
        Text(title, style: Theme.of(context).textTheme.titleLarge),
        const SizedBox(height: AppSpacing.xs),
        _message(context),
        if (actionLabel case final String label) ..._action(label),
      ],
    );
  }

  Widget _message(BuildContext context) {
    return Text(
      message,
      textAlign: TextAlign.center,
      style: Theme.of(
        context,
      ).textTheme.bodyMedium?.copyWith(color: AppPalette.mutedForeground),
    );
  }

  List<Widget> _action(String label) {
    return [
      const SizedBox(height: AppSpacing.lg),
      ElevatedButton(onPressed: onAction, child: Text(label)),
    ];
  }
}
