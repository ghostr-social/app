import 'dart:async';
import 'dart:developer';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_surface_view.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player/video_player.dart';

class VideoPlayerPlaybackPort implements VideoPlaybackPort {
  const VideoPlayerPlaybackPort();

  @override
  Widget buildSurface({
    required VideoMediaSource media,
    required bool isActive,
  }) {
    return _VideoPlayerSurface(media: media, isActive: isActive);
  }
}

class _VideoPlayerSurface extends StatefulWidget {
  const _VideoPlayerSurface({
    required this.media,
    required this.isActive,
  });

  final VideoMediaSource media;
  final bool isActive;

  @override
  State<_VideoPlayerSurface> createState() => _VideoPlayerSurfaceState();
}

class _VideoPlayerSurfaceState extends State<_VideoPlayerSurface> {
  VideoPlayerController? _controller;
  bool _hasError = false;
  int _remoteIndex = 0;

  @override
  void initState() {
    super.initState();
    _loadController();
  }

  @override
  void didUpdateWidget(covariant _VideoPlayerSurface oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.media.debugLabel != widget.media.debugLabel) {
      _hasError = false;
      _remoteIndex = 0;
      _disposeController();
      _loadController();
      return;
    }
    _syncPlayback();
  }

  @override
  void dispose() {
    _disposeController();
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
      if (_tryFallback(controller)) return;
      _rejectController(controller);
    }
  }

  VideoPlayerController _createController() {
    if (widget.media.isLocal) {
      return VideoPlayerController.file(File(widget.media.localPath!));
    }
    return VideoPlayerController.networkUrl(
      Uri.parse(widget.media.remoteUrls[_remoteIndex]),
    );
  }

  bool _tryFallback(VideoPlayerController controller) {
    if (!mounted || _controller != controller) {
      unawaited(controller.dispose());
      return true;
    }
    if (widget.media.isLocal ||
        _remoteIndex + 1 >= widget.media.remoteUrls.length) {
      return false;
    }
    _controller = null;
    _remoteIndex += 1;
    unawaited(controller.dispose());
    unawaited(_loadController());
    return true;
  }

  Future<void> _acceptController(VideoPlayerController controller) async {
    if (!mounted || _controller != controller) {
      unawaited(controller.dispose());
      return;
    }
    await _applyPlayback(controller);
    if (!mounted || _controller != controller) return;
    setState(() {});
  }

  void _rejectController(VideoPlayerController controller) {
    if (!mounted || _controller != controller) {
      unawaited(controller.dispose());
      return;
    }
    setState(() {
      _controller = null;
      _hasError = true;
    });
    unawaited(controller.dispose());
  }

  void _retry() {
    setState(() {
      _hasError = false;
      _remoteIndex = 0;
    });
    _loadController();
  }

  void _disposeController() {
    final controller = _controller;
    _controller = null;
    if (controller != null) unawaited(controller.dispose());
  }

  void _syncPlayback() {
    final controller = _controller;
    if (controller == null || !controller.value.isInitialized) {
      return;
    }
    unawaited(_guardPlayback(controller));
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
      _rejectController(controller);
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
}
