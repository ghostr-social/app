part of 'ffi_player_preparation_feedback_port.dart';

final class _FfiPlayerPreparationAttempt implements PlayerPreparationAttempt {
  _FfiPlayerPreparationAttempt(
    this._owner,
    this._authority,
    this._attempt,
    this.nativeToken,
  );

  final FfiPlayerPreparationFeedbackPort _owner;
  final PlaybackAssetAuthority _authority;
  final BigInt _attempt;
  @override
  final PlayerPreparationAttemptToken nativeToken;
  var _sequence = BigInt.zero;
  var _terminal = false;
  var _pluginReady = false;
  var _nativeFrameSeen = false;
  var _frameReported = false;
  var _begun = false;

  @override
  void begin() {
    if (_terminal || _begun) return;
    _begun = true;
    _emit(FfiPlayerPreparationState.initializing);
  }

  @override
  void initialized() {
    if (_terminal || _pluginReady) return;
    _pluginReady = true;
    _emit(FfiPlayerPreparationState.initialized);
    _reportLatchedFrame();
  }

  @override
  void firstFrameRendered() {
    if (_terminal || _nativeFrameSeen) return;
    _nativeFrameSeen = true;
    _reportLatchedFrame();
  }

  @override
  void failed(PlayerPreparationFailureKind failure) {
    if (_terminal) return;
    _terminal = true;
    _emit(FfiPlayerPreparationState.failed, failureKind: failure.name);
  }

  @override
  void release() {
    if (_terminal) return;
    _terminal = true;
    _emit(FfiPlayerPreparationState.released);
  }

  void _emit(FfiPlayerPreparationState state, {String? failureKind}) {
    _sequence += BigInt.one;
    _owner._send(
      _owner._report(
        _authority,
        _attempt,
        _sequence,
        state,
        failureKind: failureKind,
      ),
    );
  }

  void _reportLatchedFrame() {
    if (!_pluginReady || !_nativeFrameSeen || _frameReported) return;
    _frameReported = true;
    _emit(FfiPlayerPreparationState.firstFrameRendered);
  }
}
