import 'dart:async';
import 'dart:developer';

import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_inventory/domain/hls_playback_gateway_port.dart';
import 'package:ghostr/features/video_inventory/domain/hls_playback_lease.dart';
import 'package:ghostr/src/rust/video/ffi_models.dart';
import 'package:ghostr/src/rust/video/native_gateway.dart';

typedef RustHlsSessionAcquirer =
    Future<FfiHlsPlaybackSession> Function({
      required List<String> sourceUrls,
      String? deliveryId,
      String? representationId,
      BigInt? assetRevision,
    });
typedef RustHlsSessionReleaser =
    Future<bool> Function({required String sessionId});

final class FfiHlsPlaybackGateway implements HlsPlaybackGatewayPort {
  const FfiHlsPlaybackGateway({
    RustHlsSessionAcquirer acquireSession = _acquireNativeSession,
    RustHlsSessionReleaser releaseSession = ffiReleaseHlsPlayback,
  }) : _acquireSession = acquireSession,
       _releaseSession = releaseSession;

  final RustHlsSessionAcquirer _acquireSession;
  final RustHlsSessionReleaser _releaseSession;

  @override
  Future<HlsPlaybackLease> acquire(HlsPlaybackRequest request) async {
    final expected = request.expectedAuthority;
    final session = await _acquireSession(
      sourceUrls: request.sourceUrls.map((url) => url.toString()).toList(),
      deliveryId: expected?.deliveryId.value,
      representationId: expected?.representationId.value,
      assetRevision: expected?.assetRevision.value,
    );
    try {
      final authority = _authority(session);
      if (authority != expected) {
        throw const FormatException('Native HLS authority mismatch.');
      }
      final media = ProxiedHlsVideoMediaSource(session.playbackUrl);
      return HlsPlaybackLease(
        deliveryId: request.deliveryId,
        authority: authority,
        media: media,
        onReleased: () => unawaited(_releaseSafely(session.sessionId)),
      );
    } on Object {
      await _releaseSafely(session.sessionId);
      rethrow;
    }
  }

  Future<void> _releaseSafely(String sessionId) async {
    try {
      final released = await _releaseSession(sessionId: sessionId);
      if (!released) _logReleaseFailure(sessionId);
    } on Object catch (error, stackTrace) {
      log(
        'Secure HLS session release failed.',
        name: 'ghostr.video.hls',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }
}

Future<FfiHlsPlaybackSession> _acquireNativeSession({
  required List<String> sourceUrls,
  String? deliveryId,
  String? representationId,
  BigInt? assetRevision,
}) {
  final authority = _nativeAuthority(
    deliveryId,
    representationId,
    assetRevision,
  );
  return ffiAcquireHlsPlayback(authority: authority, sourceUrls: sourceUrls);
}

FfiHlsPreparedAssetAuthority? _nativeAuthority(
  String? deliveryId,
  String? representationId,
  BigInt? assetRevision,
) {
  if (deliveryId == null && representationId == null && assetRevision == null) {
    return null;
  }
  if (deliveryId == null || representationId == null || assetRevision == null) {
    throw const FormatException('Incomplete prepared HLS authority.');
  }
  return FfiHlsPreparedAssetAuthority(
    deliveryId: deliveryId,
    representationId: representationId,
    assetRevision: assetRevision,
  );
}

HlsPlaybackAuthority? _authority(FfiHlsPlaybackSession session) {
  final deliveryId = session.deliveryId;
  final representationId = session.representationId;
  final assetRevision = session.assetRevision;
  if (deliveryId == null && representationId == null && assetRevision == null) {
    return null;
  }
  if (deliveryId == null || representationId == null || assetRevision == null) {
    throw const FormatException('Incomplete native HLS authority.');
  }
  return HlsPlaybackAuthority(
    deliveryId: PlaybackDeliveryId.parse(deliveryId),
    representationId: VideoRepresentationId.parse(representationId),
    assetRevision: HlsPlaybackAssetRevision.parse(assetRevision),
  );
}

void _logReleaseFailure(String sessionId) {
  log(
    'Secure HLS session was already unavailable: $sessionId',
    name: 'ghostr.video.hls',
  );
}
