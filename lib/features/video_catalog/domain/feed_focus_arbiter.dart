import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';

/// Gives the one delivery focus sink to exactly one visible feed surface.
final class FeedFocusArbiter {
  FeedFocusArbiter(this._sink);

  final FeedFocusPort _sink;
  _ArbiterLease? _active;

  FeedFocusLease openLease() => _ArbiterLease(this);

  void _activate(_ArbiterLease lease) {
    if (identical(_active, lease)) return;
    _active = lease;
    final focus = lease._lastFocus;
    if (focus != null) {
      _sink.focusChanged(focus);
    }
  }

  void _deactivate(_ArbiterLease lease) {
    if (!identical(_active, lease)) return;
    _active = null;
  }

  void _write(_ArbiterLease lease, FeedFocus focus) {
    lease._lastFocus = focus;
    if (identical(_active, lease)) _sink.focusChanged(focus);
  }
}

final class _ArbiterLease implements FeedFocusLease {
  _ArbiterLease(this._owner);

  final FeedFocusArbiter _owner;
  var _released = false;
  FeedFocus? _lastFocus;

  @override
  void activate() {
    if (!_released) _owner._activate(this);
  }

  @override
  void deactivate() => _owner._deactivate(this);

  @override
  void focusChanged(FeedFocus focus) {
    if (!_released) _owner._write(this, focus);
  }

  @override
  void release() {
    if (_released) return;
    _released = true;
    _lastFocus = null;
    _owner._deactivate(this);
  }
}
