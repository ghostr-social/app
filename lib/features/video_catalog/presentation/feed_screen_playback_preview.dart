part of 'feed_screen.dart';

extension _FeedScreenPlaybackPreview on _FeedScreenState {
  bool _previewScroll(ScrollNotification notification, FeedLoaded state) {
    if (notification.depth != 0 || notification.metrics is! PageMetrics) {
      return false;
    }
    if (notification is ScrollEndNotification) {
      _showPreview(state, null);
    } else if (notification is ScrollUpdateNotification &&
        notification.dragDetails != null) {
      final metrics = notification.metrics;
      final origin = state.activeIndex * metrics.viewportDimension;
      final delta = metrics.pixels - origin;
      if (delta.abs() >= kTouchSlop) {
        _showPreview(state, state.activeIndex + delta.sign.toInt());
      }
    }
    return false;
  }

  void _showPreview(FeedLoaded state, int? index) {
    if (!_isVisible || _memoryConstrained) return;
    if (!_playbackPreview.show(state.roster, index)) return;
    WidgetsBinding.instance.addPostFrameCallback(
      (_) => _refreshPlaybackPreview(),
    );
  }

  void _commitPageNavigation(FeedLoaded state, int index) {
    _playbackPreview.committed(state.activeIndex, index);
    context.read<FeedCubit>().pageChanged(index);
  }

  int? _warmPageIndex(FeedLoaded state) {
    if (!_isVisible || _memoryConstrained) return null;
    return _playbackPreview.neighbor(state.roster);
  }
}
