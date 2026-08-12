import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_swipe_physics.dart';

class FeedPageView extends StatefulWidget {
  const FeedPageView({
    required this.itemCount,
    required this.onPageChanged,
    required this.itemBuilder,
    super.key,
  });

  final int itemCount;
  final ValueChanged<int> onPageChanged;
  final IndexedWidgetBuilder itemBuilder;

  @override
  State<FeedPageView> createState() => _FeedPageViewState();
}

class _FeedPageViewState extends State<FeedPageView> {
  final _controller = PageController();
  final _gesture = FeedSwipeGesture();
  late final _physics = FeedSwipePhysics(gesture: _gesture);
  int? _activePointer;

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

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }
}
