part of 'video_player_playback_port.dart';

extension _VideoPlayerSurfacePreparationFeedback on _VideoPlayerSurfaceState {
  PlayerPreparationAttempt? _preparePreparation() {
    final authority = _playbackAuthority;
    return authority == null
        ? null
        : widget.preparationFeedback.prepare(authority);
  }

  void _claimPreparation(
    VideoPlayerController controller,
    PlayerPreparationAttempt? attempt,
  ) {
    _resetPresentationEvidence();
    _preparationAttempt = attempt;
    final token = attempt?.nativeToken;
    if (token != null) {
      _firstFrameRegistration = widget.renderedFirstFrames.register(
        token,
        () => _handleNativeFrame(controller, attempt),
      );
    }
    attempt?.begin();
  }

  void _handleNativeFrame(
    VideoPlayerController controller,
    PlayerPreparationAttempt? attempt,
  ) {
    if (!_ownsController(controller) ||
        !identical(_preparationAttempt, attempt)) {
      return;
    }
    attempt?.firstFrameRendered();
    _nativeFrameObserved = true;
    _schedulePresentedFrame();
  }

  void _schedulePresentedFrame() {
    final controller = _controller;
    final attempt = _preparationAttempt;
    final session = _playbackSession;
    if (!_canSchedulePresentation(controller, attempt, session)) return;
    _presentationScheduled = true;
    final version = _presentationVersion;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _reportPresentedFrame(controller!, attempt!, session!, version);
    });
    WidgetsBinding.instance.scheduleFrame();
  }

  bool _canSchedulePresentation(
    VideoPlayerController? controller,
    PlayerPreparationAttempt? attempt,
    PlaybackSession? session,
  ) {
    return _nativeFrameObserved &&
        !_presentationScheduled &&
        !_presentationReported &&
        _controllerPresented &&
        widget.isActive &&
        _isObserving &&
        controller != null &&
        attempt != null &&
        session != null;
  }

  void _reportPresentedFrame(
    VideoPlayerController controller,
    PlayerPreparationAttempt attempt,
    PlaybackSession session,
    int version,
  ) {
    if (version != _presentationVersion) return;
    _presentationScheduled = false;
    if (!_ownsPresentedFrame(controller, attempt, session)) return;
    _presentationReported = true;
    widget.telemetry.presented(session);
  }

  bool _ownsPresentedFrame(
    VideoPlayerController controller,
    PlayerPreparationAttempt attempt,
    PlaybackSession session,
  ) {
    return _ownsController(controller) &&
        widget.isActive &&
        _isObserving &&
        identical(_preparationAttempt, attempt) &&
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
    _firstFrameRegistration?.release();
    _firstFrameRegistration = null;
    _preparationAttempt?.failed(failure);
  }

  void _releasePreparation() {
    _firstFrameRegistration?.release();
    _firstFrameRegistration = null;
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

  void _adoptRequestAuthority() {
    final authority = widget.request.authority;
    final media = _playbackMedia;
    if (authority == null ||
        media is! ProxiedProgressiveVideoMediaSource ||
        !_proxyMatches(media, authority)) {
      return;
    }
    _playbackAuthority = authority;
  }

  void _acceptRefreshedAuthority(ProxiedProgressiveVideoMediaSource refreshed) {
    final authority = _playbackAuthority;
    if (authority == null || !_proxyMatches(refreshed, authority)) {
      _playbackAuthority = null;
    }
  }
}

bool _proxyMatches(
  ProxiedProgressiveVideoMediaSource media,
  PlaybackAssetAuthority authority,
) {
  final query = media.playbackUri.queryParameters;
  return query['id'] == authority.deliveryId.value &&
      query['cap'] == authority.assetId.value;
}
