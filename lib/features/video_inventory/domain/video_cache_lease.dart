import 'package:ghostr/core/media/video_media_source.dart';

final class VideoCacheLease {
  VideoCacheLease(
    VideoMediaSource media,
    void Function() onReleased,
  ) : _state = _VideoCacheLeaseState(media, onReleased) {
    if (!media.isLocal) {
      throw ArgumentError.value(media, 'media', 'A local video is required.');
    }
  }

  VideoCacheLease._retained(this._state) {
    _state.retain();
  }

  final _VideoCacheLeaseState _state;
  bool _released = false;

  VideoMediaSource get media => _state.media;

  VideoCacheLease retain() {
    if (_released) {
      throw StateError('A released video lease cannot be retained.');
    }
    return VideoCacheLease._retained(_state);
  }

  void release() {
    if (_released) return;
    _released = true;
    _state.release();
  }
}

final class _VideoCacheLeaseState {
  _VideoCacheLeaseState(this.media, this._onReleased);

  final VideoMediaSource media;
  final void Function() _onReleased;
  int _references = 1;

  void retain() => _references += 1;

  void release() {
    _references -= 1;
    if (_references == 0) _onReleased();
  }
}
