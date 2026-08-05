import 'dart:developer';

import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/progressive_playback_gateway_port.dart';

sealed class GatewayPlaybackState {
  const GatewayPlaybackState();
}

final class GatewayPlaybackPreparing extends GatewayPlaybackState {
  const GatewayPlaybackPreparing();
}

final class GatewayPlaybackReady extends GatewayPlaybackState {
  const GatewayPlaybackReady(this.media);

  final ProxiedProgressiveVideoMediaSource media;
}

final class GatewayPlaybackFailed extends GatewayPlaybackState {
  const GatewayPlaybackFailed();
}

/// Owns progressive gateway resolution and ignores superseded answers.
final class GatewayPlaybackCubit extends Cubit<GatewayPlaybackState> {
  GatewayPlaybackCubit(this._gateway, this._media)
      : super(const GatewayPlaybackPreparing());

  final ProgressivePlaybackGatewayPort _gateway;
  VideoMediaSource _media;
  int _requestVersion = 0;

  Future<void> load(VideoMediaSource media) {
    _media = media;
    final version = ++_requestVersion;
    _prepare();
    return _resolve(media, version);
  }

  Future<void> retry() => load(_media);

  Future<void> _resolve(VideoMediaSource media, int version) async {
    try {
      final resolved = await _gateway.resolve(media);
      if (_accepts(version)) emit(GatewayPlaybackReady(resolved));
    } on Object catch (error, stackTrace) {
      _logGatewayFailure(error, stackTrace);
      if (_accepts(version)) emit(const GatewayPlaybackFailed());
    }
  }

  void _prepare() {
    if (state is! GatewayPlaybackPreparing) {
      emit(const GatewayPlaybackPreparing());
    }
  }

  bool _accepts(int version) => !isClosed && version == _requestVersion;

  @override
  Future<void> close() {
    _requestVersion += 1;
    return super.close();
  }
}

void _logGatewayFailure(Object error, StackTrace stackTrace) {
  log(
    'Progressive gateway resolution failed.',
    name: 'ghostr.video.gateway',
    error: error,
    stackTrace: stackTrace,
  );
}
