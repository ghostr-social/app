import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_inventory/domain/hls_playback_gateway_port.dart';
import 'package:ghostr/features/video_inventory/domain/hls_playback_lease.dart';
import 'package:ghostr/platform/media/ffi_hls_playback_gateway.dart';

final class WarpHlsPlaybackGatewayProbe implements HlsPlaybackGatewayPort {
  WarpHlsPlaybackGatewayProbe({
    HlsPlaybackGatewayPort delegate = const FfiHlsPlaybackGateway(),
  }) : _delegate = delegate;

  final HlsPlaybackGatewayPort _delegate;
  final acquisitions = <WarpHlsLeaseEvidence>[];

  @override
  Future<HlsPlaybackLease> acquire(HlsPlaybackRequest request) async {
    final upstream = await _delegate.acquire(request);
    final evidence = WarpHlsLeaseEvidence(
      request.deliveryId,
      request.representationId,
      request.expectedAuthority,
      request.sourceUrls,
      upstream.media.playbackUri,
      upstream.authority,
    );
    acquisitions.add(evidence);
    return HlsPlaybackLease(
      deliveryId: upstream.deliveryId,
      authority: upstream.authority,
      media: upstream.media,
      onReleased: () {
        evidence.released = true;
        upstream.release();
      },
    );
  }

  List<WarpHlsLeaseEvidence> activeFor(PlaybackDeliveryId deliveryId) {
    return acquisitions
        .where((item) => item.deliveryId == deliveryId && !item.released)
        .toList(growable: false);
  }
}

final class WarpHlsLeaseEvidence {
  WarpHlsLeaseEvidence(
    this.deliveryId,
    this.representationId,
    this.expectedAuthority,
    this.sourceUrls,
    this.playbackUri,
    this.authority,
  );

  final PlaybackDeliveryId deliveryId;
  final VideoRepresentationId representationId;
  final HlsPlaybackAuthority? expectedAuthority;
  final List<Uri> sourceUrls;
  final Uri playbackUri;
  final HlsPlaybackAuthority? authority;
  bool released = false;

  String get sessionId => playbackUri.pathSegments[1];
}
