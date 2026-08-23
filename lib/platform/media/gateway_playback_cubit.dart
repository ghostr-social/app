import 'dart:developer';

import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/core/media/playback_asset_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/prepared_progressive_playback.dart';
import 'package:ghostr/core/media/video_media_cache_identity.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_inventory/domain/progressive_playback_gateway_port.dart';

sealed class GatewayPlaybackState {
  const GatewayPlaybackState();
}

final class GatewayPlaybackPreparing extends GatewayPlaybackState {
  const GatewayPlaybackPreparing();
}

final class GatewayPlaybackReady extends GatewayPlaybackState {
  const GatewayPlaybackReady(this.origin, this.media);

  final VideoMediaSource origin;
  final ProxiedProgressiveVideoMediaSource media;
}

final class GatewayPlaybackFailed extends GatewayPlaybackState {
  const GatewayPlaybackFailed();
}

/// Owns progressive gateway resolution and ignores superseded answers.
final class GatewayPlaybackCubit extends Cubit<GatewayPlaybackState> {
  GatewayPlaybackCubit(this._gateway) : super(const GatewayPlaybackPreparing());

  final ProgressivePlaybackGatewayPort _gateway;
  VideoMediaSource? _origin;
  VideoRepresentationId? _originRepresentation;
  int _requestVersion = 0;

  Future<void> load(
    VideoMediaSource origin, {
    PreparedProgressivePlayback? prepared,
  }) {
    final representation = _representationOf(origin);
    if (_sameOrigin(origin, representation)) return _adopt(origin, prepared);
    _origin = origin;
    _originRepresentation = representation;
    return prepared == null
        ? _startResolution(origin)
        : _publishPrepared(origin, prepared);
  }

  Future<void> retry() {
    final origin = _origin;
    return origin == null ? Future<void>.value() : _startResolution(origin);
  }

  Future<void> _adopt(
    VideoMediaSource origin,
    PreparedProgressivePlayback? prepared,
  ) {
    if (prepared == null) {
      return Future<void>.value();
    }
    if (!prepared.matches(origin)) {
      throw ArgumentError.value(prepared, 'prepared', 'Origin mismatch.');
    }
    final current = state;
    if (current is GatewayPlaybackReady &&
        current.media.playbackAssetId == prepared.authority.assetId) {
      return Future<void>.value();
    }
    return _publishPrepared(origin, prepared);
  }

  Future<void> _publishPrepared(
    VideoMediaSource origin,
    PreparedProgressivePlayback prepared,
  ) {
    if (!prepared.matches(origin)) {
      throw ArgumentError.value(prepared, 'prepared', 'Origin mismatch.');
    }
    _requestVersion += 1;
    emit(GatewayPlaybackReady(origin, prepared.media));
    return Future<void>.value();
  }

  Future<void> _startResolution(VideoMediaSource origin) {
    final version = ++_requestVersion;
    _prepare();
    return _resolve(origin, version);
  }

  Future<void> _resolve(VideoMediaSource origin, int version) async {
    try {
      final resolved = await _gateway.resolve(origin);
      if (!_accepts(version)) return;
      _validateResolvedDelivery(origin, resolved);
      emit(GatewayPlaybackReady(origin, resolved));
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

  bool _sameOrigin(
    VideoMediaSource origin,
    VideoRepresentationId? representation,
  ) {
    return representation != null &&
        _originRepresentation == representation &&
        _origin?.inventoryPlaybackIdentity == origin.inventoryPlaybackIdentity;
  }

  @override
  Future<void> close() {
    _requestVersion += 1;
    return super.close();
  }
}

void _validateResolvedDelivery(
  VideoMediaSource origin,
  ProxiedProgressiveVideoMediaSource resolved,
) {
  if (origin.playbackDeliveryId != resolved.playbackDeliveryId) {
    throw StateError('Gateway changed playback delivery.');
  }
}

VideoRepresentationId? _representationOf(VideoMediaSource media) {
  try {
    return VideoRepresentationId.forMedia(media);
  } on ArgumentError {
    return null;
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
