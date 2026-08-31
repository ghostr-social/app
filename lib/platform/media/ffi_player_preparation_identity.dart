part of 'ffi_player_preparation_feedback_port.dart';

final class _PlayerPreparationIdentity {
  const _PlayerPreparationIdentity({
    required this.postId,
    required this.representationId,
    required this.assetId,
  });

  factory _PlayerPreparationIdentity.progressive(
    PlaybackAssetAuthority authority,
  ) {
    return _PlayerPreparationIdentity(
      postId: authority.deliveryId.value,
      representationId: authority.representationId.value,
      assetId: authority.assetId.value,
    );
  }

  factory _PlayerPreparationIdentity.hls(HlsPlaybackAuthority authority) {
    final revision = authority.assetRevision.value;
    if (revision > _maximumU64) {
      throw ArgumentError.value(revision, 'assetRevision', 'Exceeds u64.');
    }
    return _PlayerPreparationIdentity(
      postId: authority.deliveryId.value,
      representationId: authority.representationId.value,
      assetId: 'hls-v1:$revision',
    );
  }

  final String postId;
  final String representationId;
  final String assetId;
}

final _maximumU64 = BigInt.parse('18446744073709551615');
