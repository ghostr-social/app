import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_asset_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_representation_id.dart';

void main() {
  test('playback asset authority has complete value equality', () {
    final first = authority('a');
    final equal = authority('a');
    final different = authority('b');

    expect(first, equal);
    expect(first.hashCode, equal.hashCode);
    expect(first, isNot(different));
    expect(first.assetId, equal.assetId);
    expect(first.assetId.hashCode, equal.assetId.hashCode);
    expect(first.assetId, isNot(different.assetId));
  });
}

PlaybackAssetAuthority authority(String suffix) {
  return PlaybackAssetAuthority(
    deliveryId: PlaybackDeliveryId.parse('delivery-$suffix'),
    representationId: VideoRepresentationId.parse(suffix * 64),
    assetId: PlaybackAssetId.parse(suffix * 43),
  );
}
