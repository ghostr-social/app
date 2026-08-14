part of 'rust_feed_remote_source.dart';

/// Owns one passive Dart listener and the native feed lease behind it.
final class RustFeedRemoteWatcher {
  RustFeedRemoteWatcher(this._source, this._spec, this._viewer) {
    _controller = StreamController<RemoteVideoSnapshot>(
      onListen: () => unawaited(_start()),
      onCancel: _stop,
    );
  }

  final RustFeedRemoteSource _source;
  final FfiFeedSpec _spec;
  final NostrPublicKeyHex? _viewer;
  late final StreamController<RemoteVideoSnapshot> _controller;
  StreamSubscription<RustFeedPage>? _updates;
  RustFeedSession? _session;
  bool _pinned = false;
  bool _stopped = false;

  Stream<RemoteVideoSnapshot> get stream => _controller.stream;

  Future<void> _start() async {
    try {
      final session = await _source._opened(_spec, _viewer);
      _session = session;
      _source._sessions.pin(session);
      _pinned = true;
      if (_stopped) {
        await _release(session);
        return;
      }
      _updates = session.watchPages().listen(
        _publish,
        onError: _failed,
        onDone: _ended,
      );
    } on Object catch (error, stackTrace) {
      _failed(error, stackTrace);
    }
  }

  void _publish(RustFeedPage page) {
    if (_stopped) return;
    _controller.add(
      RemoteVideoSnapshot(
        revision: page.revision,
        phase: _source._phase(page.stage),
        posts: _source._mapped(page.posts, null),
      ),
    );
  }

  void _failed(Object error, StackTrace stackTrace) {
    if (_stopped) return;
    _controller.addError(_source._failure(error, stackTrace), stackTrace);
    unawaited(_finished());
  }

  void _ended() {
    if (_stopped) return;
    _failed(rustFeedFailure, StackTrace.current);
  }

  Future<void> _finished() async {
    await _stop();
    await _controller.close();
  }

  Future<void> _stop() async {
    if (_stopped) return;
    _stopped = true;
    await _updates?.cancel();
    final session = _session;
    if (session == null) return;
    await _release(session);
  }

  Future<void> _release(RustFeedSession session) async {
    if (!_pinned) return;
    _pinned = false;
    if (await _source._sessions.unpin(session)) {
      await _source._sessions.retire(session);
    }
  }
}
