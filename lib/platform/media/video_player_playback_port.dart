import 'dart:async';
import 'dart:developer';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:ghostr/core/media/video_media_cache_identity.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_surface_view.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player/video_player.dart';

part 'video_player_controller_lifecycle.dart';
part 'video_player_media_controller.dart';

class VideoPlayerPlaybackPort implements VideoPlaybackPort {
  const VideoPlayerPlaybackPort({
    VideoPlayerControllerDisposer controllerDisposer =
        disposeVideoPlayerController,
  }) : _controllerDisposer = controllerDisposer;

  final VideoPlayerControllerDisposer _controllerDisposer;

  @override
  Widget buildSurface({
    required VideoMediaSource media,
    required bool isActive,
    void Function()? onPlaybackMediaReleased,
  }) {
    return _VideoPlayerSurface(
      key: ValueKey(media.inventoryPlaybackIdentity),
      media: media,
      isActive: isActive,
      onPlaybackMediaReleased: onPlaybackMediaReleased,
      controllerDisposer: _controllerDisposer,
    );
  }
}

class _VideoPlayerSurface extends StatefulWidget {
  const _VideoPlayerSurface({
    super.key,
    required this.media,
    required this.isActive,
    required this.onPlaybackMediaReleased,
    required this.controllerDisposer,
  });

  final VideoMediaSource media;
  final bool isActive;
  final void Function()? onPlaybackMediaReleased;
  final VideoPlayerControllerDisposer controllerDisposer;

  @override
  State<_VideoPlayerSurface> createState() => _VideoPlayerSurfaceState();
}

class _VideoPlayerSurfaceState extends State<_VideoPlayerSurface> {
  VideoPlayerController? _controller;
  late final _lifecycle =
      _VideoPlayerControllerLifecycle(widget.controllerDisposer);
  late bool _hasError = !_isPlayableMedia(widget.media);
  bool _isClosing = false;

  @override
  void initState() {
    super.initState();
    if (!_hasError) _startLoad();
  }

  @override
  void didUpdateWidget(covariant _VideoPlayerSurface oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.isActive != widget.isActive) _syncPlayback();
  }

  @override
  void dispose() {
    _isClosing = true;
    final released = widget.onPlaybackMediaReleased;
    final disposal = _disposeCurrentController();
    if (disposal != null) _lifecycle.track(disposal);
    unawaited(_releaseWhenClosed(released));
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return VideoPlayerSurfaceView(
      controller: _controller,
      hasError: _hasError,
      onRetry: _retry,
    );
  }

  Future<void> _loadController() async {
    final controller = _createController();
    _controller = controller;
    try {
      await controller.setLooping(true);
      await controller.initialize();
      await _acceptController(controller);
    } on Object catch (error, stackTrace) {
      log(
        'Video player initialization failed.',
        name: 'ghostr.video.player',
        error: error,
        stackTrace: stackTrace,
      );
      await _rejectController(controller);
    }
  }

  VideoPlayerController _createController() {
    return _videoPlayerController(widget.media);
  }

  Future<void> _acceptController(VideoPlayerController controller) async {
    if (!_ownsController(controller)) {
      await _disposeSafely(controller);
      return;
    }
    await _applyPlayback(controller);
    if (!_ownsController(controller)) return;
    setState(() {});
  }

  bool _ownsController(VideoPlayerController controller) {
    return !_isClosing && mounted && _controller == controller;
  }

  Future<void> _rejectController(VideoPlayerController controller) async {
    if (!_isClosing && mounted && _controller == controller) {
      setState(() {
        _controller = null;
        _hasError = true;
      });
    } else if (_controller == controller) {
      _controller = null;
    }
    await _disposeSafely(controller);
  }

  void _retry() {
    if (!_isPlayableMedia(widget.media)) return;
    setState(() {
      _hasError = false;
    });
    _startLoad();
  }

  Future<void>? _disposeCurrentController() {
    final controller = _controller;
    _controller = null;
    return controller == null ? null : _disposeSafely(controller);
  }

  void _syncPlayback() {
    final controller = _controller;
    if (controller == null || !controller.value.isInitialized) {
      return;
    }
    _lifecycle.track(_guardPlayback(controller));
  }

  Future<void> _guardPlayback(VideoPlayerController controller) async {
    try {
      await _applyPlayback(controller);
    } on Object catch (error, stackTrace) {
      log(
        'Video playback command failed.',
        name: 'ghostr.video.player',
        error: error,
        stackTrace: stackTrace,
      );
      await _rejectController(controller);
    }
  }

  Future<void> _applyPlayback(VideoPlayerController controller) async {
    if (widget.isActive) {
      await controller.play();
      return;
    }
    await controller.pause();
    await controller.seekTo(Duration.zero);
  }

  void _startLoad() => _lifecycle.track(_loadController());

  Future<void> _releaseWhenClosed(void Function()? released) async {
    await _lifecycle.close();
    released?.call();
  }

  Future<void> _disposeSafely(VideoPlayerController controller) {
    return _lifecycle.dispose(controller);
  }
}
