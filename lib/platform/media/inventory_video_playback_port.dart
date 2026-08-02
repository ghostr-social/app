import 'dart:async';
import 'dart:developer';

import 'package:flutter/material.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_priority.dart';
import 'package:ghostr/features/video_inventory/domain/video_inventory_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

class InventoryVideoPlaybackPort implements VideoPlaybackPort {
  const InventoryVideoPlaybackPort({
    required VideoPlaybackPort delegate,
    required VideoInventoryPort inventory,
  })  : _delegate = delegate,
        _inventory = inventory;

  final VideoPlaybackPort _delegate;
  final VideoInventoryPort _inventory;

  @override
  Widget buildSurface({
    required VideoMediaSource media,
    required bool isActive,
  }) {
    return _InventoryVideoSurface(
      delegate: _delegate,
      inventory: _inventory,
      media: media,
      isActive: isActive,
    );
  }
}

class _InventoryVideoSurface extends StatefulWidget {
  const _InventoryVideoSurface({
    required this.delegate,
    required this.inventory,
    required this.media,
    required this.isActive,
  });

  final VideoPlaybackPort delegate;
  final VideoInventoryPort inventory;
  final VideoMediaSource media;
  final bool isActive;

  @override
  State<_InventoryVideoSurface> createState() => _InventoryVideoSurfaceState();
}

class _InventoryVideoSurfaceState extends State<_InventoryVideoSurface> {
  late VideoMediaSource _playbackMedia = widget.media;
  VideoMediaSource? _cachedMedia;
  late bool _isPreparing = !widget.media.isLocal;
  int _requestVersion = 0;

  @override
  void initState() {
    super.initState();
    _requestCache(_priority);
  }

  @override
  void didUpdateWidget(covariant _InventoryVideoSurface oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.media.debugLabel != widget.media.debugLabel) {
      _resetMedia();
      return;
    }
    if (oldWidget.isActive != widget.isActive) _syncActivity();
  }

  @override
  Widget build(BuildContext context) {
    if (!widget.isActive && _isPreparing && !_playbackMedia.isLocal) {
      return const ColoredBox(
        color: AppPalette.videoLoadingBackground,
        child: Center(child: Text('Preparing next video')),
      );
    }
    return widget.delegate.buildSurface(
      media: _playbackMedia,
      isActive: widget.isActive,
    );
  }

  VideoCachePriority get _priority => widget.isActive
      ? VideoCachePriority.foreground
      : VideoCachePriority.background;

  void _resetMedia() {
    _requestVersion += 1;
    _playbackMedia = widget.media;
    _cachedMedia = null;
    _isPreparing = !widget.media.isLocal;
    _requestCache(_priority);
  }

  void _syncActivity() {
    if (widget.isActive && _cachedMedia != null) {
      setState(() => _playbackMedia = _cachedMedia!);
      return;
    }
    if (widget.isActive && _cachedMedia == null) {
      _requestCache(VideoCachePriority.foreground);
    }
  }

  void _requestCache(VideoCachePriority priority) {
    if (widget.media.isLocal) return;
    final version = ++_requestVersion;
    unawaited(_loadCachedMedia(version, priority));
  }

  Future<void> _loadCachedMedia(
    int version,
    VideoCachePriority priority,
  ) async {
    try {
      final media = await widget.inventory.cache(widget.media, priority);
      if (!_isCurrent(version)) return;
      _acceptCachedMedia(media);
    } catch (error, stackTrace) {
      log('Video cache preparation failed.',
          name: 'ghostr.video', error: error, stackTrace: stackTrace);
      if (!_isCurrent(version)) return;
      _acceptCachedMedia(widget.media);
    }
  }

  bool _isCurrent(int version) => mounted && version == _requestVersion;

  void _acceptCachedMedia(VideoMediaSource media) {
    setState(() {
      _isPreparing = false;
      _cachedMedia = media.isLocal ? media : null;
      if (!widget.isActive && media.isLocal) _playbackMedia = media;
    });
  }
}
