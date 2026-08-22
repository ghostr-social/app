part of 'video_player_playback_port.dart';

enum _HandoffCommandOutcome { completed, disposed, cancelled }

enum _HandoffCommandEffect { mute, pause, play, unmute }

enum _ControllerTeardownOutcome { proven, unproven }

typedef _UnsafePlaybackAbandon = void Function();

final class _HandoffControllerState {
  _HandoffActivation? _activation;
  final Completer<_ControllerSettlement> retirement = Completer();
  _ControllerSettlement? teardown;
  var retiring = false;
  var _muteRepairDirty = false;
  var _muteRepairScheduled = false;
  final Set<Future<void>> _unsafeCommands = {};

  _HandoffActivation beginActivation(
    _PlaybackOwnership ownsPlayback,
    _UnsafePlaybackAbandon onUnsafeAbandon,
  ) {
    cancelActivation();
    return _activation = _HandoffActivation(ownsPlayback, onUnsafeAbandon);
  }

  _HandoffActivation? get liveActivation {
    final activation = _activation;
    return activation?.isLive == true ? activation : null;
  }

  bool requestMuteRepair() {
    _muteRepairDirty = true;
    if (_muteRepairScheduled) return false;
    return _muteRepairScheduled = true;
  }

  bool takeMuteRepair() {
    if (!_muteRepairDirty) return false;
    _muteRepairDirty = false;
    return true;
  }

  bool finishMuteRepair() {
    _muteRepairScheduled = false;
    return _muteRepairDirty;
  }

  void holdUnsafeCommand(Future<void> native) {
    final settled = native.then<void>((_) {}, onError: (_, __) {});
    _unsafeCommands.add(settled);
    unawaited(settled.whenComplete(() => _unsafeCommands.remove(settled)));
  }

  Future<void> waitUnsafeCommands() async {
    while (_unsafeCommands.isNotEmpty) {
      await Future.wait(_unsafeCommands.toList());
    }
  }

  void cancelActivation() {
    _activation?.cancel();
    _activation = null;
  }

  void retire(_ControllerSettlement settlement) {
    cancelActivation();
    retiring = true;
    teardown = settlement;
    if (!retirement.isCompleted) retirement.complete(settlement);
  }
}

final class _HandoffActivation {
  _HandoffActivation(this._ownsPlayback, this._onUnsafeAbandon);

  final _PlaybackOwnership _ownsPlayback;
  final _UnsafePlaybackAbandon _onUnsafeAbandon;
  final Completer<void> _cancelled = Completer();
  var _active = true;
  var _unsafeAbandoned = false;

  Future<void> get cancelled => _cancelled.future;
  bool get isLive => _active && _ownsPlayback();

  void abandonUnsafe() {
    if (_unsafeAbandoned) return;
    _unsafeAbandoned = true;
    _onUnsafeAbandon();
  }

  void cancel() {
    if (!_active) return;
    _active = false;
    _cancelled.complete();
  }
}
