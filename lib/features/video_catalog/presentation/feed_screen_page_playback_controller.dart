part of 'feed_screen.dart';

final class _FeedPagePlaybackController extends ChangeNotifier {
  Set<VideoPostId> _playbackIds = const {};
  Set<VideoPostId> _keepAliveIds = const {};

  bool plays(VideoPostId postId) => _playbackIds.contains(postId);

  bool keepsAlive(VideoPostId postId) => _keepAliveIds.contains(postId);

  void synchronize({
    required Set<VideoPostId> playbackIds,
    required Set<VideoPostId> keepAliveIds,
  }) {
    if (setEquals(_playbackIds, playbackIds) &&
        setEquals(_keepAliveIds, keepAliveIds)) {
      return;
    }
    _playbackIds = Set.unmodifiable(playbackIds);
    _keepAliveIds = Set.unmodifiable(keepAliveIds);
    notifyListeners();
  }
}

final class _PlaybackFeedPage extends StatefulWidget {
  const _PlaybackFeedPage({
    required this.controller,
    required this.postId,
    required this.child,
  });

  final _FeedPagePlaybackController controller;
  final VideoPostId postId;
  final Widget child;

  @override
  State<_PlaybackFeedPage> createState() => _PlaybackFeedPageState();
}

final class _PlaybackFeedPageState extends State<_PlaybackFeedPage>
    with AutomaticKeepAliveClientMixin {
  @override
  bool get wantKeepAlive => widget.controller.keepsAlive(widget.postId);

  @override
  void initState() {
    super.initState();
    widget.controller.addListener(_synchronize);
  }

  @override
  void didUpdateWidget(covariant _PlaybackFeedPage oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (identical(oldWidget.controller, widget.controller)) return;
    oldWidget.controller.removeListener(_synchronize);
    widget.controller.addListener(_synchronize);
    _synchronize();
  }

  void _synchronize() {
    if (!mounted) return;
    updateKeepAlive();
    setState(() {});
  }

  @override
  Widget build(BuildContext context) {
    super.build(context);
    if (widget.controller.plays(widget.postId)) return widget.child;
    return ColoredBox(key: ValueKey(widget.postId.value), color: Colors.black);
  }

  @override
  void dispose() {
    widget.controller.removeListener(_synchronize);
    super.dispose();
  }
}
