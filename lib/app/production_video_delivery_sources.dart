import 'dart:developer';

import 'package:ghostr/core/media/video_playback_capabilities.dart';
import 'package:ghostr/features/video_catalog/data/ffi_video_remote_source.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_snapshot.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_inventory/domain/hls_playback_gateway_port.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';

RemoteVideoSource nativeRemoteVideoSource(
  VideoGatewayStartResult result,
  NostrVideoSnapshot snapshot,
) {
  return switch (result) {
    VideoGatewayStarted() => FfiVideoRemoteSource(
        snapshotLoader: snapshot.read,
      ),
    VideoGatewayFailed(:final message) => _reportedFailure(message),
  };
}

HlsPlaybackGatewayPort? activeHlsGateway({
  required VideoGatewayStartResult result,
  required HlsPlaybackGatewayPort gateway,
  required VideoPlaybackCapabilities capabilities,
}) {
  if (result is! VideoGatewayStarted || !capabilities.supportsHls) {
    return null;
  }
  return gateway;
}

DisabledRemoteVideoSource _reportedFailure(String message) {
  log(message, name: 'ghostr.gateway');
  return const DisabledRemoteVideoSource(
    'The embedded Nostr gateway is unavailable.',
  );
}
