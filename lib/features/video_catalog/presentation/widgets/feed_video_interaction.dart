import 'package:flutter/material.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

typedef FeedVideoSurfaceBuilder = Widget Function(VideoPlaybackMode mode);

class FeedVideoInteraction extends StatefulWidget {
  const FeedVideoInteraction({
    required this.isActive,
    required this.surfaceBuilder,
    required this.onOpenMenu,
    required this.overlay,
    super.key,
  });

  final bool isActive;
  final FeedVideoSurfaceBuilder surfaceBuilder;
  final VoidCallback onOpenMenu;
  final Widget overlay;

  @override
  State<FeedVideoInteraction> createState() => _FeedVideoInteractionState();
}

class _FeedVideoInteractionState extends State<FeedVideoInteraction> {
  VideoPlaybackMode _mode = VideoPlaybackMode.normal;
  VideoPlaybackMode? _modeBeforeAcceleration;

  @override
  void didUpdateWidget(covariant FeedVideoInteraction oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.isActive && !widget.isActive) _resetMode();
  }

  @override
  Widget build(BuildContext context) {
    return Semantics(
      container: true,
      explicitChildNodes: true,
      label: _mode == VideoPlaybackMode.paused ? 'Play video' : 'Pause video',
      hint: 'Tap the video center to toggle playback. Long press for actions.',
      button: true,
      enabled: widget.isActive,
      onTap: widget.isActive ? _togglePlayback : null,
      onLongPress: widget.isActive ? widget.onOpenMenu : null,
      child: Listener(
        onPointerCancel: _handlePointerCancel,
        child: LayoutBuilder(builder: _buildGestureSurface),
      ),
    );
  }

  Widget _buildGestureSurface(
    BuildContext context,
    BoxConstraints constraints,
  ) {
    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      excludeFromSemantics: true,
      onTapUp: (details) =>
          _handleTap(details.localPosition, constraints.maxWidth),
      onLongPressStart: (details) => _handleLongPress(details.localPosition),
      onLongPressEnd: (_) => _restorePreviousMode(),
      onLongPressCancel: _restorePreviousMode,
      child: Stack(
        fit: StackFit.expand,
        children: [widget.surfaceBuilder(_mode), widget.overlay, _feedback],
      ),
    );
  }

  void _handleTap(Offset position, double width) {
    if (_isCenter(position.dx, width)) _togglePlayback();
  }

  void _handleLongPress(Offset position) {
    if (position.dx <= AppSize.feedGestureEdge) {
      _accelerate();
      return;
    }
    widget.onOpenMenu();
  }

  void _togglePlayback() {
    if (!widget.isActive) return;
    setState(() {
      _mode = _mode == VideoPlaybackMode.paused
          ? VideoPlaybackMode.normal
          : VideoPlaybackMode.paused;
      _modeBeforeAcceleration = null;
    });
  }

  void _accelerate() {
    if (!widget.isActive || _mode == VideoPlaybackMode.accelerated) return;
    setState(() {
      _modeBeforeAcceleration = _mode;
      _mode = VideoPlaybackMode.accelerated;
    });
  }

  void _restorePreviousMode() {
    final previous = _modeBeforeAcceleration;
    if (previous == null) return;
    setState(() {
      _mode = previous;
      _modeBeforeAcceleration = null;
    });
  }

  void _handlePointerCancel(PointerCancelEvent event) {
    _restorePreviousMode();
  }

  void _resetMode() {
    _mode = VideoPlaybackMode.normal;
    _modeBeforeAcceleration = null;
  }

  bool _isCenter(double x, double width) {
    return x > AppSize.feedGestureEdge && x < width - AppSize.feedGestureEdge;
  }

  Widget get _feedback => switch (_mode) {
    VideoPlaybackMode.paused => const _PausedFeedback(),
    VideoPlaybackMode.accelerated => const _AcceleratedFeedback(),
    VideoPlaybackMode.normal => const SizedBox.shrink(),
  };
}

class _PausedFeedback extends StatelessWidget {
  const _PausedFeedback();

  @override
  Widget build(BuildContext context) {
    return const IgnorePointer(
      child: Center(
        child: DecoratedBox(
          decoration: BoxDecoration(
            color: AppPalette.videoControlBackground,
            shape: BoxShape.circle,
          ),
          child: Padding(
            padding: EdgeInsets.all(AppSpacing.sm),
            child: Icon(
              Icons.play_arrow_rounded,
              color: AppPalette.foreground,
              size: AppSize.feedPlaybackIcon,
            ),
          ),
        ),
      ),
    );
  }
}

class _AcceleratedFeedback extends StatelessWidget {
  const _AcceleratedFeedback();

  @override
  Widget build(BuildContext context) {
    return Align(
      alignment: const Alignment(0, -0.55),
      child: Semantics(
        container: true,
        excludeSemantics: true,
        liveRegion: true,
        label: 'Playing at 2x speed',
        child: IgnorePointer(
          child: Material(
            color: AppPalette.videoControlBackground,
            borderRadius: BorderRadius.circular(AppRadius.control),
            child: Padding(
              padding: const EdgeInsets.symmetric(
                horizontal: AppSpacing.md,
                vertical: AppSpacing.xs,
              ),
              child: Text(
                '2×',
                style: Theme.of(context).textTheme.titleMedium?.copyWith(
                  color: AppPalette.foreground,
                  shadows: AppShadow.videoOverlay,
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}
