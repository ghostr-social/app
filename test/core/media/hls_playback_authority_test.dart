import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_representation_id.dart';

void main() {
  test(
    'HLS playback authority is positive and has complete value equality',
    () {
      final first = _authority(BigInt.one);
      final equal = _authority(BigInt.one);
      final revised = _authority(BigInt.two);

      expect(first, equal);
      expect(first.hashCode, equal.hashCode);
      expect(first, isNot(revised));
      expect(first.assetRevision, equal.assetRevision);
      expect(first.assetRevision.hashCode, equal.assetRevision.hashCode);
      expect(
        () => HlsPlaybackAssetRevision.parse(BigInt.zero),
        throwsArgumentError,
      );
    },
  );
}

HlsPlaybackAuthority _authority(BigInt revision) {
  return HlsPlaybackAuthority(
    deliveryId: PlaybackDeliveryId.parse('post-A'),
    representationId: VideoRepresentationId.parse('a' * 64),
    assetRevision: HlsPlaybackAssetRevision.parse(revision),
  );
}
