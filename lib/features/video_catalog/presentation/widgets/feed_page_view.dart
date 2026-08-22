import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_swipe_physics.dart';

final class FeedPageModel {
  FeedPageModel({required Iterable<Key> keys, this.activePage = 0})
    : keys = List<Key>.unmodifiable(keys) {
    if (this.keys.isEmpty && activePage == 0) return;
    RangeError.checkValidIndex(activePage, this.keys, 'activePage');
    if (this.keys.toSet().length != this.keys.length) {
      throw ArgumentError.value(this.keys, 'keys', 'must be unique');
    }
  }

  final List<Key> keys;
  final int activePage;

  Key? get activeKey => keys.isEmpty ? null : keys[activePage];
}

class FeedPageView extends StatefulWidget {
  const FeedPageView({
    required this.model,
    required this.onPageChanged,
    required this.itemBuilder,
    super.key,
  });

  final FeedPageModel model;
  final bool Function(int index) onPageChanged;
  final IndexedWidgetBuilder itemBuilder;

  @override
  State<FeedPageView> createState() => _FeedPageViewState();
}

class _FeedPageViewState extends State<FeedPageView> {
  late final _controller = PageController(initialPage: widget.model.activePage);
  final _gesture = FeedSwipeGesture();
  late final _physics = FeedSwipePhysics(gesture: _gesture);
  late Key? _reportedKey;
  List<Key>? _gestureKeys;
  Key? _candidateKey;
  int? _activePointer;
  var _repositionPending = false;

  @override
  void initState() {
    super.initState();
    _reportedKey = widget.model.activeKey;
  }

  @override
  void didUpdateWidget(covariant FeedPageView oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (_activePageChanged(oldWidget)) _requestReposition();
  }

  bool _activePageChanged(FeedPageView oldWidget) {
    return oldWidget.model.activePage != widget.model.activePage ||
        oldWidget.model.activeKey != widget.model.activeKey;
  }

  @override
  Widget build(BuildContext context) {
    return Listener(
      onPointerDown: _beginGesture,
      onPointerUp: _endGesture,
      onPointerCancel: _cancelGesture,
      child: NotificationListener<ScrollEndNotification>(
        onNotification: _scrollEnded,
        child: PageView.builder(
          controller: _controller,
          scrollDirection: Axis.vertical,
          dragStartBehavior: DragStartBehavior.down,
          physics: _physics,
          pageSnapping: false,
          allowImplicitScrolling: true,
          itemCount: widget.model.keys.length,
          onPageChanged: _pageChanged,
          itemBuilder: _buildPage,
          findChildIndexCallback: _pageForKey,
        ),
      ),
    );
  }

  Widget _buildPage(BuildContext context, int index) {
    return KeyedSubtree(
      key: widget.model.keys[index],
      child: widget.itemBuilder(context, index),
    );
  }

  int? _pageForKey(Key key) {
    final index = widget.model.keys.indexOf(key);
    return index < 0 ? null : index;
  }

  void _pageChanged(int index) {
    _candidateKey = widget.model.keys[index];
  }

  bool _scrollEnded(ScrollEndNotification notification) {
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
    _completeGesture(targetKey, currentTarget);
  }

  void _completeGesture(Key? targetKey, int? currentTarget) {
    _activePointer = null;
    _gestureKeys = null;
    _candidateKey = null;
    final accepted = currentTarget == null || _commit(targetKey);
    if (_repositionPending && accepted) _requestReposition();
  }

  Key? _gestureKey(int index) {
    final keys = _gestureKeys;
    if (keys == null || index < 0 || index >= keys.length) return null;
    return keys[index];
  }

  bool _commit(Key? key) {
    if (key == null || key == _reportedKey) return true;
    final index = _pageForKey(key);
    if (index == null) return true;
    if (!widget.onPageChanged(index)) {
      _requestReposition();
      return false;
    }
    _reportedKey = key;
    return true;
  }

  void _cancelGesture(PointerCancelEvent event) {
    if (_activePointer != event.pointer) return;
    _activePointer = null;
    _gestureKeys = null;
    _candidateKey = null;
    _gesture.reset();
    if (_repositionPending) _requestReposition();
  }

  void _requestReposition() {
    _repositionPending = true;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) _reposition();
    });
  }

  void _reposition() {
    if (!_repositionPending || _activePointer != null) return;
    final keys = widget.model.keys;
    if (keys.isEmpty) {
      _repositionPending = false;
      return;
    }
    if (!_controller.hasClients) {
      _requestReposition();
      return;
    }
    final target = widget.model.activePage.clamp(0, keys.length - 1);
    _repositionPending = false;
    if (_controller.page?.round() != target) _controller.jumpToPage(target);
    _commit(keys[target]);
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }
}
