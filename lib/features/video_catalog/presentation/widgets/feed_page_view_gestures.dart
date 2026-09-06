part of 'feed_page_view.dart';

extension _FeedPageGestures on _FeedPageViewState {
  void _pageChanged(int index) {
    _candidateKey = widget.model.keys[index];
  }

  bool _scrollEnded(ScrollEndNotification notification) {
    if (notification.depth != 0 || notification.metrics is! PageMetrics) {
      return false;
    }
    if (_activePointer != null) return false;
    final candidate = _candidateKey;
    _candidateKey = null;
    _commit(candidate);
    return false;
  }

  void _beginGesture(PointerDownEvent event) {
    if (_activePointer != null) return;
    _activePointer = event.pointer;
    _gestureKeys = List<Key>.of(widget.model.keys);
    _candidateKey = null;
    _gesture.begin();
  }

  void _endGesture(PointerUpEvent event) {
    if (_activePointer != event.pointer) return;
    final target = _gesture.targetPage;
    final targetKey = target == null ? null : _gestureKey(target);
    final currentTarget = targetKey == null ? null : _pageForKey(targetKey);
    if (target != null && currentTarget == null) {
      _gesture.targetPage = widget.model.activePage;
    }
    _gesture.end();
    if (currentTarget != null && _controller.hasClients) {
      _controller.jumpToPage(currentTarget);
    }
    _activePointer = null;
    _gestureKeys = null;
    _candidateKey = null;
    if (currentTarget != null) _commit(targetKey);
  }

  Key? _gestureKey(int index) {
    final keys = _gestureKeys;
    if (keys == null || index < 0 || index >= keys.length) return null;
    return keys[index];
  }

  void _commit(Key? key) {
    if (key == null || key == _reportedKey) return;
    final index = _pageForKey(key);
    if (index == null) return;
    _reportedKey = key;
    widget.onPageChanged(index);
  }

  void _cancelGesture(PointerCancelEvent event) {
    if (_activePointer != event.pointer) return;
    _activePointer = null;
    _gestureKeys = null;
    _candidateKey = null;
    _gesture.reset();
    if (_controller.hasClients) {
      _controller.jumpToPage(widget.model.activePage);
    }
  }
}
