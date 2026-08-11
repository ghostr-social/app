import 'package:flutter/foundation.dart';
import 'package:video_player/video_player.dart';

/// Forwards plugin values while owning listener attachment and teardown.
final class VideoPlayerValueListener {
  VideoPlayerValueListener({required this.onValueChanged});

  final ValueChanged<VideoPlayerValue> onValueChanged;

  ValueListenable<VideoPlayerValue>? _listenable;

  void attach(ValueListenable<VideoPlayerValue> listenable) {
    detach();
    _listenable = listenable;
    listenable.addListener(_handleValue);
    _handleValue();
  }

  void detach() {
    _listenable?.removeListener(_handleValue);
    _listenable = null;
  }

  void _handleValue() {
    final value = _listenable?.value;
    if (value == null) return;
    onValueChanged(value);
  }
}
