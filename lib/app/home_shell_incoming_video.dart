part of 'home_shell.dart';

extension _HomeShellIncomingVideo on _HomeShellState {
  void _initializeIncomingVideos() {
    _composeCubit = widget.controllers.compose();
    _composeCubit.bindPreviewRelease(
      widget.controllers.incomingVideoSharePort.release,
    );
    _incomingVideoShares = widget.controllers.incomingVideoSharePort.events
        .listen(_handleIncomingVideo);
  }

  void _handleIncomingVideo(IncomingVideoShareEvent event) {
    switch (event) {
      case IncomingVideoShareReady(:final media):
        final request = ++_latestIncomingVideoRequest;
        _pendingIncomingVideos[request] = media;
        unawaited(_openIncomingVideo(media, request));
      case IncomingVideoShareFailure(:final message):
        _showIncomingVideoFailure(message);
    }
  }

  Future<void> _openIncomingVideo(SelectedMedia media, int request) async {
    final recovery = _composeRecovery;
    if (recovery != null) await recovery;
    if (!await _waitForIncomingSlot(media, request)) return;
    if (!await _acknowledgeIncomingVideo(media)) {
      _pendingIncomingVideos.remove(request);
      return;
    }
    if (!await _waitForIncomingSlot(media, request)) return;
    _composeRecoveryStarted = true;
    _pendingIncomingVideos.remove(request);
    if (!_composeCubit.acceptSharedVideo(media)) {
      await _releaseIncomingVideo(media);
      return;
    }
    _retainUnmountedIncomingVideo(media);
    _dismissCoveredRoutes();
    _activateTab(HomeTab.create);
  }

  void _retainUnmountedIncomingVideo(SelectedMedia media) {
    final replaced = _unmountedIncomingVideos.values
        .where((previous) => previous.path != media.path)
        .toList(growable: false);
    _unmountedIncomingVideos
      ..clear()
      ..[media.path] = media;
    for (final previous in replaced) {
      unawaited(_releaseIncomingVideo(previous));
    }
  }

  void _claimIncomingVideoPreview(SelectedMedia media) {
    _unmountedIncomingVideos.remove(media.path);
  }

  Future<bool> _waitForIncomingSlot(SelectedMedia media, int request) async {
    if (!await _isCurrentIncomingRequest(media, request)) return false;
    if (!await _waitUntilComposerIdle()) return false;
    return _isCurrentIncomingRequest(media, request);
  }

  Future<bool> _isCurrentIncomingRequest(
    SelectedMedia media,
    int request,
  ) async {
    if (mounted && request == _latestIncomingVideoRequest) return true;
    if (_pendingIncomingVideos.remove(request) != null) {
      await _releaseIncomingVideo(media);
    }
    return false;
  }

  Future<bool> _waitUntilComposerIdle() async {
    while (mounted && _composeCubit.state.isBusy) {
      try {
        await _composeCubit.stream.firstWhere((state) => !state.isBusy);
      } on StateError {
        return false;
      }
    }
    return mounted;
  }

  Future<void> _releaseIncomingVideo(SelectedMedia media) async {
    if (media.source != MediaPickSource.externalShare) return;
    try {
      await widget.controllers.incomingVideoSharePort.release(media);
    } on Object {
      return;
    }
  }

  Future<bool> _acknowledgeIncomingVideo(SelectedMedia media) async {
    try {
      await widget.controllers.incomingVideoSharePort.acknowledge(media);
      return true;
    } on Object {
      await _releaseIncomingVideo(media);
      _showIncomingVideoFailure('Could not open the shared video.');
      return false;
    }
  }

  void _showIncomingVideoFailure(String message) {
    if (!mounted) return;
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(SnackBar(content: Text(message)));
  }

  void _recoverComposeDraft() {
    if (_composeRecoveryStarted) return;
    _composeRecoveryStarted = true;
    _composeRecovery = _composeCubit.recoverLostVideo();
  }

  void _disposeIncomingVideos() {
    unawaited(_incomingVideoShares.cancel());
    final pending = _pendingIncomingVideos.values.toList(growable: false);
    _pendingIncomingVideos.clear();
    final unmounted = _unmountedIncomingVideos.values.toList(growable: false);
    _unmountedIncomingVideos.clear();
    for (final media in [...pending, ...unmounted]) {
      unawaited(_releaseIncomingVideo(media));
    }
    unawaited(_closeComposerWhenIdle());
  }

  Future<void> _closeComposerWhenIdle() async {
    while (_composeCubit.state.isPublishing) {
      try {
        await _composeCubit.stream.firstWhere((state) => !state.isPublishing);
      } on StateError {
        break;
      }
    }
    await _composeCubit.close();
  }
}
