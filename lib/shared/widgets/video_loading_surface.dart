import 'package:flutter/material.dart';
import 'package:ghostr/core/media/inline_blurhash.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';
import 'package:ghostr/shared/widgets/inline_blurhash_preview.dart';
import 'package:ghostr/shared/widgets/loading_panel.dart';

/// Shared loading treatment for every video delivery mode.
class VideoLoadingSurface extends StatelessWidget {
  const VideoLoadingSurface({required this.label, this.preview, super.key});

  final String label;
  final InlineBlurHash? preview;

  @override
  Widget build(BuildContext context) {
    final descriptor = preview;
    return ColoredBox(
      color: AppPalette.videoLoadingBackground,
      child: Stack(
        fit: StackFit.expand,
        children: [
          if (descriptor != null) InlineBlurHashPreview(descriptor: descriptor),
          if (descriptor != null)
            const ColoredBox(color: AppPalette.videoControlBackground),
          LoadingPanel(label: label),
        ],
      ),
    );
  }
}
