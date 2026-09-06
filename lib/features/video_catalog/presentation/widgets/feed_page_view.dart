import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_state.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_swipe_physics.dart';

part 'feed_page_view_gestures.dart';
part 'feed_page_view_overlay.dart';

final class FeedPageModel {
  FeedPageModel({
    required Iterable<Key> keys,
    required this.rosterRevision,
    this.activePage = 0,
  }) : keys = List<Key>.unmodifiable(keys) {
    if (this.keys.isEmpty && activePage == 0) return;
    RangeError.checkValidIndex(activePage, this.keys, 'activePage');
    if (this.keys.toSet().length != this.keys.length) {
      throw ArgumentError.value(this.keys, 'keys', 'must be unique');
    }
  }

  final List<Key> keys;
  final FeedRosterRevision rosterRevision;
  final int activePage;

  Key? get activeKey => keys.isEmpty ? null : keys[activePage];
}

class FeedPageView extends StatefulWidget {
  const FeedPageView({
    required this.model,
    required this.onPageChanged,
    required this.itemBuilder,
    this.overlay,
    super.key,
  });

  final FeedPageModel model;
  final ValueChanged<int> onPageChanged;
  final IndexedWidgetBuilder itemBuilder;
  final Widget? overlay;

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

  @override
  void initState() {
    super.initState();
    _reportedKey = widget.model.activeKey;
  }

  @override
  void didUpdateWidget(covariant FeedPageView oldWidget) {
    super.didUpdateWidget(oldWidget);
    final sameRoster = identical(
      oldWidget.model.rosterRevision,
      widget.model.rosterRevision,
    );
    final sameActive = oldWidget.model.activeKey == widget.model.activeKey;
    if (sameRoster && sameActive && _activePointer != null) return;
    final reported = _reportedKey;
    final pending = reported != widget.model.activeKey;
    final retained = reported != null && _pageForKey(reported) != null;
    if (sameRoster && sameActive && pending && retained) return;
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
      child: NotificationListener<ScrollEndNotification>(
        onNotification: _scrollEnded,
        child: Stack(
          fit: StackFit.expand,
          children: [
            PageView.builder(
              controller: _controller,
              scrollDirection: Axis.vertical,
              dragStartBehavior: DragStartBehavior.down,
              physics: _physics,
              pageSnapping: false,
              allowImplicitScrolling: true,
              scrollCacheExtent: _transportRescuePageCache,
              itemCount: widget.model.keys.length,
              onPageChanged: _pageChanged,
              itemBuilder: _buildPage,
              findChildIndexCallback: _pageForKey,
            ),
            if (widget.overlay case final overlay?)
              _FeedPageScrollOverlay(controller: _controller, child: overlay),
          ],
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

  void _reposition() {
    if (widget.model.keys.isEmpty) return;
    if (!_controller.hasClients) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) _reposition();
      });
      return;
    }
    final target = widget.model.activePage.clamp(
      0,
      widget.model.keys.length - 1,
    );
    _reportedKey = widget.model.keys[target];
    if (_controller.page?.round() != target) _controller.jumpToPage(target);
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }
}

const _transportRescuePageCache = ScrollCacheExtent.viewport(3.0);
