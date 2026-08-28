part of 'video_player_playback_port.dart';

extension _VideoPlayerSurfacePreparationFeedback on _VideoPlayerSurfaceState {
  PlayerPreparationAttempt? _preparePreparation() {
    final authority = _playbackAuthority;
    final media = _playbackMedia;
    if (authority == null ||
        media is! ProxiedProgressiveVideoMediaSource ||
        !_proxyMatches(media, authority)) {
      return null;
    }
    return widget.preparationFeedback.prepare(authority);
  }

  void _schedulePresentedFrame() {
    final controller = _controller;
    final frameAttempt = _firstFrameAttempt;
    final session = _playbackSession;
    if (!_canSchedulePresentation(controller, frameAttempt, session)) return;
    _presentationScheduled = true;
    final version = _presentationVersion;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _reportPresentedFrame(controller!, frameAttempt!, session!, version);
    });
    WidgetsBinding.instance.scheduleFrame();
  }

  bool _canSchedulePresentation(
    VideoPlayerController? controller,
    RenderedFirstFrameAttempt? frameAttempt,
    PlaybackSession? session,
  ) {
    return _nativeFrameObserved &&
        !_presentationScheduled &&
        !_presentationReported &&
        _controllerPresented &&
        widget.isActive &&
        _isObserving &&
        controller != null &&
        frameAttempt != null &&
        session != null;
  }

  void _reportPresentedFrame(
    VideoPlayerController controller,
    RenderedFirstFrameAttempt frameAttempt,
    PlaybackSession session,
    int version,
  ) {
    if (version != _presentationVersion) return;
    _presentationScheduled = false;
    if (!_ownsPresentedFrame(controller, frameAttempt, session)) return;
    _presentationReported = true;
    widget.telemetry.presented(session);
  }

  bool _ownsPresentedFrame(
    VideoPlayerController controller,
    RenderedFirstFrameAttempt frameAttempt,
    PlaybackSession session,
  ) {
    return _ownsController(controller) &&
        widget.isActive &&
        _isObserving &&
        identical(_firstFrameAttempt, frameAttempt) &&
        _playbackSession == session &&
        _nativeFrameObserved &&
        !_presentationReported;
  }

  void _finishPluginInitialization() {
    _controllerPresented = true;
    _preparationAttempt?.initialized();
    _schedulePresentedFrame();
  }

  void _failPreparation(PlayerPreparationFailureKind failure) {
    final frameAttempt = _firstFrameAttempt;
    _firstFrameAttempt = null;
    if (frameAttempt != null) _releaseFrameAttemptSafely(frameAttempt);
    _preparationAttempt?.failed(failure);
  }

  void _releasePreparation() {
    final frameAttempt = _firstFrameAttempt;
    _firstFrameAttempt = null;
    if (frameAttempt != null) _releaseFrameAttemptSafely(frameAttempt);
    _preparationAttempt?.release();
    _preparationAttempt = null;
    _resetPresentationEvidence();
  }

  void _resetPresentationEvidence() {
    _presentationVersion += 1;
    _nativeFrameObserved = false;
    _controllerPresented = false;
    _presentationScheduled = false;
    _presentationReported = false;
  }

  void _resetActivationPresentation() {
    _presentationVersion += 1;
    _presentationScheduled = false;
    _presentationReported = false;
  }

  void _adoptRequestAuthority() {
    final authority = widget.request.authority;
    final media = widget.media;
    if (authority == null ||
        media is! ProxiedProgressiveVideoMediaSource ||
        !_proxyMatches(media, authority)) {
      return;
    }
    final previous = _playbackAuthority;
    if (previous == null ||
        previous.deliveryId != authority.deliveryId ||
        previous.representationId != authority.representationId) {
      return;
    }
    _playbackAuthority = authority;
    if (previous.assetId == authority.assetId) return;
    _playbackMedia = media;
    _restartUninitializedPlayback();
  }

  void _restartUninitializedPlayback() {
    final controller = _controller;
    if (controller?.value.isInitialized == true && _isObserving) return;
    _cancelRecovery();
    _resetRecoveryBudget();
    if (_recoveryState != _VideoPlayerRecoveryState.ready) {
      _refresh(() => _recoveryState = _VideoPlayerRecoveryState.ready);
    }
    if (controller != null) {
      _relinquishController(controller);
      _lifecycle.track(_disposeSafely(controller));
    }
    _startLoad();
  }
}

PlaybackAssetAuthority? _renewedAuthority(
  PlaybackAssetAuthority? previous,
  ProxiedProgressiveVideoMediaSource media,
) {
  if (previous == null || media.playbackDeliveryId != previous.deliveryId) {
    return null;
  }
  return PlaybackAssetAuthority(
    deliveryId: previous.deliveryId,
    representationId: previous.representationId,
    assetId: media.playbackAssetId,
  );
}

bool _proxyMatches(
  ProxiedProgressiveVideoMediaSource media,
  PlaybackAssetAuthority authority,
) {
  return media.playbackDeliveryId == authority.deliveryId &&
      media.playbackAssetId == authority.assetId;
}
