part of 'app_update_status_panel.dart';

final class _PanelAction {
  const _PanelAction(this.label, this.onPressed);

  final String label;
  final VoidCallback? onPressed;
}

class _StatusView extends StatelessWidget {
  const _StatusView({
    required this.icon,
    required this.message,
    this.detail,
    this.detailIsError = false,
    this.progressLabel,
    this.progress,
    this.primary,
    this.secondary,
    this.outlined,
  });

  final IconData icon;
  final String message;
  final String? detail;
  final bool detailIsError;
  final String? progressLabel;
  final double? progress;
  final _PanelAction? primary;
  final _PanelAction? secondary;
  final _PanelAction? outlined;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      container: true,
      liveRegion: progressLabel != null,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: _children(context),
      ),
    );
  }

  List<Widget> _children(BuildContext context) {
    return [
      Row(
        children: [
          Icon(icon),
          const SizedBox(width: AppSpacing.sm),
          _title(context),
        ],
      ),
      if (detail != null) ...[
        const SizedBox(height: AppSpacing.xs),
        _detail(context),
      ],
      if (progressLabel != null) ..._progressWidgets(),
      if (primary != null || secondary != null || outlined != null)
        ..._actions(),
    ];
  }

  Widget _title(BuildContext context) {
    return Expanded(
      child: Text(message, style: Theme.of(context).textTheme.titleMedium),
    );
  }

  Widget _detail(BuildContext context) {
    final text = Text(
      detail!,
      style: detailIsError
          ? TextStyle(color: Theme.of(context).colorScheme.error)
          : null,
    );
    if (!detailIsError) return text;
    return Semantics(
      container: true,
      liveRegion: true,
      label: detail,
      child: ExcludeSemantics(child: text),
    );
  }

  List<Widget> _progressWidgets() {
    return [
      const SizedBox(height: AppSpacing.sm),
      LinearProgressIndicator(
        value: progress,
        semanticsLabel: progressLabel,
        semanticsValue: progress == null
            ? null
            : '${(progress! * 100).round()}%',
      ),
    ];
  }

  List<Widget> _actions() {
    return [
      const SizedBox(height: AppSpacing.md),
      Wrap(
        spacing: AppSpacing.sm,
        children: [
          if (primary case final action?)
            FilledButton(
              onPressed: action.onPressed,
              child: Text(action.label),
            ),
          if (outlined case final action?)
            OutlinedButton(
              onPressed: action.onPressed,
              child: Text(action.label),
            ),
          if (secondary case final action?)
            TextButton(onPressed: action.onPressed, child: Text(action.label)),
        ],
      ),
    ];
  }
}
