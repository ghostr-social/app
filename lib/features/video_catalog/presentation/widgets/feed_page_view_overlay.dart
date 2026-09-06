part of 'feed_page_view.dart';

/// Gives fixed controls the same scroll position and physics as the media pages.
class _FeedPageScrollOverlay extends StatefulWidget {
  const _FeedPageScrollOverlay({required this.controller, required this.child});

  final PageController controller;
  final Widget child;

  @override
  State<_FeedPageScrollOverlay> createState() => _FeedPageScrollOverlayState();
}

class _FeedPageScrollOverlayState extends State<_FeedPageScrollOverlay> {
  Drag? _drag;

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      dragStartBehavior: DragStartBehavior.down,
      onVerticalDragStart: _start,
      onVerticalDragUpdate: _update,
      onVerticalDragEnd: _end,
      onVerticalDragCancel: _cancel,
      child: widget.child,
    );
  }

  void _start(DragStartDetails details) {
    _cancel();
    if (!widget.controller.hasClients) return;
    _drag = widget.controller.position.drag(details, _clear);
  }

  void _update(DragUpdateDetails details) => _drag?.update(details);

  void _end(DragEndDetails details) {
    _drag?.end(details);
    _clear();
  }

  void _cancel() {
    _drag?.cancel();
    _clear();
  }

  void _clear() => _drag = null;

  @override
  void dispose() {
    _cancel();
    super.dispose();
  }
}
