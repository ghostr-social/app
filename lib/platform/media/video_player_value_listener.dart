import 'package:flutter/foundation.dart';
import 'package:video_player/video_player.dart';

/// Watches a video player's [VideoPlayerValue] and reports stall and
/// error transitions so playback surfaces can re-render their loading
/// and error panels mid-stream.
final class VideoPlayerValueListener {
  VideoPlayerValueListener({required this.onStateChanged});

  final VoidCallback onStateChanged;

  ValueListenable<VideoPlayerValue>? _listenable;
  bool _isStalled = false;
  bool _hasError = false;

  /// True while an initialized player is buffering.
  bool get isStalled => _isStalled;

  /// True once the player reported a playback error.
  bool get hasError => _hasError;

  void attach(ValueListenable<VideoPlayerValue> listenable) {
    detach();
    _listenable = listenable;
    listenable.addListener(_handleValue);
    _handleValue();
  }

  void detach() {
    _listenable?.removeListener(_handleValue);
    _listenable = null;
    _isStalled = false;
    _hasError = false;
  }

  void _handleValue() {
    final value = _listenable?.value;
    if (value == null) return;
    final stalled = value.isInitialized && value.isBuffering;
    final failed = value.hasError;
    if (stalled == _isStalled && failed == _hasError) return;
    _isStalled = stalled;
    _hasError = failed;
    onStateChanged();
  }
}
