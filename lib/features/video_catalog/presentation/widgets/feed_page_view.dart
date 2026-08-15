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
  final ValueChanged<int> onPageChanged;
  final IndexedWidgetBuilder itemBuilder;

  @override
  State<FeedPageView> createState() => _FeedPageViewState();
}

class _FeedPageViewState extends State<FeedPageView> {
  late final _controller = PageController(initialPage: widget.initialPage);
  final _gesture = FeedSwipeGesture();
  late final _physics = FeedSwipePhysics(gesture: _gesture);
  int? _activePointer;

  @override
  void didUpdateWidget(covariant FeedPageView oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.initialPage == widget.initialPage) return;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) _reposition();
    });
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
        onPageChanged: widget.onPageChanged,
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
  }

  void _cancelGesture(PointerCancelEvent event) {
    if (_activePointer != event.pointer) return;
    _activePointer = null;
    _gesture.reset();
  }

  void _reposition() {
    if (widget.itemCount == 0) return;
    if (!_controller.hasClients) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) _reposition();
      });
      return;
    }
    final target = widget.initialPage.clamp(0, widget.itemCount - 1);
    if (_controller.page?.round() != target) _controller.jumpToPage(target);
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }
}
