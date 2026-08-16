import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_swipe_physics.dart';

class FeedPageView extends StatefulWidget {
  const FeedPageView({
    required this.itemCount,
    required this.onPageChanged,
    required this.itemBuilder,
    this.initialPage = 0,
    super.key,
  });

  final int itemCount;
  final int initialPage;
  final bool Function(int index) onPageChanged;
  final IndexedWidgetBuilder itemBuilder;

  @override
  State<FeedPageView> createState() => _FeedPageViewState();
}

class _FeedPageViewState extends State<FeedPageView> {
  late final _controller = PageController(initialPage: widget.initialPage);
  final _gesture = FeedSwipeGesture();
  late final _physics = FeedSwipePhysics(gesture: _gesture);
  int? _activePointer;
  var _repositionPending = false;

  @override
  void didUpdateWidget(covariant FeedPageView oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.initialPage != widget.initialPage) _requestReposition();
  }

  @override
  Widget build(BuildContext context) {
    return Listener(
      onPointerDown: _beginGesture,
      onPointerUp: _endGesture,
      onPointerCancel: _cancelGesture,
      child: PageView.builder(
        controller: _controller,
        scrollDirection: Axis.vertical,
        dragStartBehavior: DragStartBehavior.down,
        physics: _physics,
        pageSnapping: false,
        allowImplicitScrolling: true,
        itemCount: widget.itemCount,
        onPageChanged: _pageChanged,
        itemBuilder: widget.itemBuilder,
      ),
    );
  }

  void _beginGesture(PointerDownEvent event) {
    if (_activePointer != null) return;
    _activePointer = event.pointer;
    _gesture.begin();
  }

  void _endGesture(PointerUpEvent event) {
    if (_activePointer != event.pointer) return;
    _activePointer = null;
    final target = _gesture.end();
    if (target != null && _controller.hasClients) {
      _controller.jumpToPage(target);
    }
    if (_repositionPending) _requestReposition();
  }

  void _cancelGesture(PointerCancelEvent event) {
    if (_activePointer != event.pointer) return;
    _activePointer = null;
    _gesture.reset();
    if (_repositionPending) _requestReposition();
  }

  void _pageChanged(int index) {
    if (!widget.onPageChanged(index)) _rejectPageChange();
  }

  void _rejectPageChange() {
    if (!mounted) return;
    _requestReposition();
  }

  void _requestReposition() {
    _repositionPending = true;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) _reposition();
    });
  }

  void _reposition() {
    if (!_repositionPending) return;
    if (_activePointer != null) return;
    if (widget.itemCount == 0) {
      _repositionPending = false;
      return;
    }
    if (!_controller.hasClients) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) _reposition();
      });
      return;
    }
    final target = widget.initialPage.clamp(0, widget.itemCount - 1);
    _repositionPending = false;
    if (_controller.page?.round() != target) _controller.jumpToPage(target);
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }
}
