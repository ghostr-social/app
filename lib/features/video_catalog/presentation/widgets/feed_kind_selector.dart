import 'package:flutter/material.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

class FeedKindOverlay extends StatelessWidget {
  const FeedKindOverlay({
    required this.selected,
    required this.onSelected,
    required this.visible,
    required this.child,
    super.key,
  });

  final FeedKind selected;
  final ValueChanged<FeedKind> onSelected;
  final bool visible;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    if (!visible) return child;
    return Stack(
      fit: StackFit.expand,
      children: [
        child,
        Align(
          alignment: Alignment.topCenter,
          child: SafeArea(
            child: FeedKindSelector(selected: selected, onSelected: onSelected),
          ),
        ),
      ],
    );
  }
}

class FeedKindSelector extends StatelessWidget {
  const FeedKindSelector({
    required this.selected,
    required this.onSelected,
    super.key,
  });

  final FeedKind selected;
  final ValueChanged<FeedKind> onSelected;

  @override
  Widget build(BuildContext context) {
    return Material(
      color: AppPalette.surface,
      borderRadius: BorderRadius.circular(AppRadius.control),
      child: SegmentedButton<FeedKind>(
        segments: [
          for (final kind in FeedKind.values)
            ButtonSegment(value: kind, label: Text(kind.label)),
        ],
        selected: {selected},
        showSelectedIcon: false,
        onSelectionChanged: _select,
      ),
    );
  }

  void _select(Set<FeedKind> values) {
    final next = values.single;
    if (next != selected) onSelected(next);
  }
}
