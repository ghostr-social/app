import 'package:flutter/material.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

class FeedCardRepostAction extends StatefulWidget {
  const FeedCardRepostAction({
    required this.post,
    required this.onToggle,
    super.key,
  });

  final VideoPost post;
  final Future<void> Function(VideoPost post)? onToggle;

  @override
  State<FeedCardRepostAction> createState() => _FeedCardRepostActionState();
}

class _FeedCardRepostActionState extends State<FeedCardRepostAction> {
  bool _isToggling = false;
  bool _wasReposted = false;

  @override
  Widget build(BuildContext context) {
    return IconButton(
      onPressed: _canToggle ? _toggle : null,
      tooltip: _tooltip,
      isSelected: widget.post.viewerHasReposted,
      iconSize: AppSize.feedRailIcon,
      icon: _isToggling ? _progress() : _icon(),
    );
  }

  bool get _canToggle => widget.onToggle != null && !_isToggling;

  String get _tooltip {
    if (widget.onToggle == null) return 'Reposting unavailable for this video';
    return _actionWasReposted ? 'Undo repost' : 'Repost video';
  }

  Widget _progress() {
    final label = _wasReposted ? 'Removing repost' : 'Reposting video';
    return Stack(
      alignment: Alignment.center,
      children: [
        _icon(),
        SizedBox.square(
          dimension: AppSize.feedRailIcon,
          child: CircularProgressIndicator(
            strokeWidth: 2,
            semanticsLabel: label,
          ),
        ),
      ],
    );
  }

  bool get _actionWasReposted =>
      _isToggling ? _wasReposted : widget.post.viewerHasReposted;

  Widget _icon() {
    return Icon(
      Icons.repeat,
      color: widget.post.viewerHasReposted
          ? AppPalette.accentBlue
          : AppPalette.foreground,
      shadows: AppShadow.videoOverlay,
    );
  }

  Future<void> _toggle() async {
    setState(() {
      _wasReposted = widget.post.viewerHasReposted;
      _isToggling = true;
    });
    try {
      await widget.onToggle!(widget.post);
    } finally {
      if (mounted) setState(() => _isToggling = false);
    }
  }
}
