import 'package:ghostr/core/media/video_media_source.dart';

final class HlsPlaybackLease {
  HlsPlaybackLease(this.media, this._onReleased);

  final ProxiedHlsVideoMediaSource media;
  final void Function() _onReleased;
  bool _released = false;

  void release() {
    if (_released) return;
    _released = true;
    _onReleased();
  }
}
