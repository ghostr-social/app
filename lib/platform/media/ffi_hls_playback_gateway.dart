import 'dart:async';
import 'dart:developer';

import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/hls_playback_gateway_port.dart';
import 'package:ghostr/features/video_inventory/domain/hls_playback_lease.dart';
import 'package:ghostr/src/rust/video/ffi_models.dart';
import 'package:ghostr/src/rust/video/native_gateway.dart';

typedef RustHlsSessionAcquirer =
    Future<FfiHlsPlaybackSession> Function({required List<String> sourceUrls});
typedef RustHlsSessionReleaser =
    Future<bool> Function({required String sessionId});

final class FfiHlsPlaybackGateway implements HlsPlaybackGatewayPort {
  const FfiHlsPlaybackGateway({
    RustHlsSessionAcquirer acquireSession = ffiAcquireHlsPlayback,
    RustHlsSessionReleaser releaseSession = ffiReleaseHlsPlayback,
  }) : _acquireSession = acquireSession,
       _releaseSession = releaseSession;

  final RustHlsSessionAcquirer _acquireSession;
  final RustHlsSessionReleaser _releaseSession;

  @override
  Future<HlsPlaybackLease> acquire(HlsPlaybackRequest request) async {
    final session = await _acquireSession(
      sourceUrls: request.sourceUrls.map((url) => url.toString()).toList(),
    );
    try {
      final media = ProxiedHlsVideoMediaSource(session.playbackUrl);
      return HlsPlaybackLease(
        deliveryId: request.deliveryId,
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

void _logReleaseFailure(String sessionId) {
  log(
    'Secure HLS session was already unavailable: $sessionId',
    name: 'ghostr.video.hls',
  );
}
