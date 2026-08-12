import 'dart:async';
import 'dart:collection';

import 'package:ghostr/core/media/incoming_video_share.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/core/media/video_mime_type.dart';

abstract interface class IncomingVideoShareGateway {
  Stream<void> get videoAvailable;

  Future<Map<Object?, Object?>?> takePendingVideo();

  Future<void> acknowledgeVideo(String path);

  Future<void> releaseVideo(String path);

  Future<void> close();
}

/// App-lifetime adapter owned by the production dependency graph.
///
/// Its gateway notification subscription intentionally remains active while UI
/// listeners come and go so native deliveries can be drained after sign-in.
final class AndroidIncomingVideoSharePort implements IncomingVideoSharePort {
  AndroidIncomingVideoSharePort(this._gateway) {
    _events = StreamController<IncomingVideoShareEvent>.broadcast(
      onListen: _onListen,
      sync: true,
    );
    _gatewaySubscription = _gateway.videoAvailable.listen(_onVideoAvailable);
  }

  static const _failureMessage = 'Could not open the shared video.';

  final IncomingVideoShareGateway _gateway;
  late final StreamSubscription<void> _gatewaySubscription;
  late final StreamController<IncomingVideoShareEvent> _events;
  final Queue<IncomingVideoShareEvent> _retainedEvents = Queue();
  Future<void>? _activeDrain;
  Future<void>? _closeFuture;
  bool _isClosed = false;
  bool _initialDrainRequested = false;
  bool _isDraining = false;
  int _pendingDrains = 0;

  @override
  Stream<IncomingVideoShareEvent> get events => _events.stream;

  @override
  Future<void> acknowledge(SelectedMedia media) {
    if (media.source != MediaPickSource.externalShare) return Future.value();
    return _gateway.acknowledgeVideo(media.path);
  }

  @override
  Future<void> release(SelectedMedia media) {
    if (media.source != MediaPickSource.externalShare) return Future.value();
    return _gateway.releaseVideo(media.path);
  }

  @override
  Future<void> close() => _closeFuture ??= _close();

  Future<void> _close() async {
    _isClosed = true;
    await _gatewaySubscription.cancel();
    await _activeDrain;
    await _releaseRetainedVideos();
    await _events.close();
    await _gateway.close();
  }

  void _onListen() {
    if (_isClosed) return;
    _replayRetainedEvents();
    if (!_initialDrainRequested) {
      _initialDrainRequested = true;
      if (_pendingDrains == 0) _pendingDrains = 1;
    }
    _startDrain();
  }

  void _onVideoAvailable(void _) {
    _requestDrain();
  }

  void _requestDrain() {
    _pendingDrains += 1;
    _startDrain();
  }

  void _startDrain() {
    if (_isClosed || _pendingDrains == 0 || !_events.hasListener) return;
    if (_isDraining) return;
    final drain = _drainPending();
    _activeDrain = drain;
    unawaited(drain);
  }

  Future<void> _drainPending() async {
    _isDraining = true;
    try {
      while (!_isClosed && _pendingDrains > 0 && _events.hasListener) {
        _pendingDrains -= 1;
        await _emitPendingVideo();
      }
    } finally {
      _isDraining = false;
      _startDrain();
    }
  }

  Future<void> _emitPendingVideo() async {
    try {
      final payload = await _gateway.takePendingVideo();
      if (payload == null) return;
      final event = _mapPayload(payload);
      if (_isClosed || event is IncomingVideoShareFailure) {
        await _releasePayload(payload);
      }
      if (_isClosed) return;
      _publish(event);
    } on Object {
      _publish(const IncomingVideoShareFailure(_failureMessage));
    }
  }

  Future<void> _releasePayload(Map<Object?, Object?> payload) async {
    final path = payload['path'];
    if (path is! String || path.startsWith('content://')) return;
    await _tryReleasePath(path);
  }

  Future<void> _releaseRetainedVideos() async {
    while (_retainedEvents.isNotEmpty) {
      final event = _retainedEvents.removeFirst();
      if (event is IncomingVideoShareReady) {
        await _tryReleasePath(event.media.path);
      }
    }
  }

  Future<void> _tryReleasePath(String path) async {
    try {
      await _gateway.releaseVideo(path);
    } on Object {
      // The user-safe failure remains actionable even if cleanup also fails.
    }
  }

  void _publish(IncomingVideoShareEvent event) {
    if (_isClosed) return;
    if (_events.hasListener) {
      _events.add(event);
    } else {
      _retainedEvents.addLast(event);
    }
  }

  void _replayRetainedEvents() {
    if (_isClosed || !_events.hasListener || _retainedEvents.isEmpty) return;
    final event = _retainedEvents.first;
    scheduleMicrotask(() {
      if (_isClosed || !_events.hasListener || _retainedEvents.isEmpty) return;
      _events.add(event);
      _retainedEvents.removeFirst();
      _replayRetainedEvents();
    });
  }

  IncomingVideoShareEvent _mapPayload(Map<Object?, Object?> payload) {
    try {
      final path = _field(payload, 'path');
      final label = _field(payload, 'label');
      final mimeType = VideoMimeType.tryParse(_field(payload, 'mimeType'));
      if (path.startsWith('content://') || mimeType == null) {
        throw const FormatException('Invalid shared video.');
      }
      return IncomingVideoShareReady(
        SelectedMedia(
          path: path,
          source: MediaPickSource.externalShare,
          label: label,
          mimeType: mimeType,
        ),
      );
    } on Object {
      return const IncomingVideoShareFailure(_failureMessage);
    }
  }

  String _field(Map<Object?, Object?> payload, String key) {
    final value = payload[key];
    if (value is! String) throw const FormatException('Invalid field.');
    return value;
  }
}
