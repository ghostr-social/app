part of 'warp_feed_playback_journey.dart';

bool warpNativeStoreHistoryIsValid(
  Iterable<VideoDeliverySnapshot> history,
  PlaybackDeliveryId deliveryId,
  BigInt expectedTotal,
) {
  var hasExactTotal = false;
  PlaybackAssetAuthority? authority;
  for (final snapshot in history.where(
    (snapshot) => snapshot.deliveryId == deliveryId,
  )) {
    if (!_isValidNativeStoreSnapshot(snapshot, expectedTotal)) return false;
    if (snapshot.totalBytes == expectedTotal) hasExactTotal = true;
    if (snapshot.bytesPresent == BigInt.zero) continue;
    final observed = snapshot.authority;
    if (observed == null || observed.deliveryId != deliveryId) return false;
    if (authority != null && authority != observed) return false;
    authority = observed;
  }
  return hasExactTotal && authority != null;
}

bool _isValidNativeStoreSnapshot(
  VideoDeliverySnapshot snapshot,
  BigInt expectedTotal,
) {
  final total = snapshot.totalBytes;
  return snapshot.phase != VideoDeliveryPhase.failed &&
      snapshot.bytesPresent >= BigInt.zero &&
      snapshot.bytesPresent <= expectedTotal &&
      (total == null || total == expectedTotal);
}

Nip01Event _eventForOriginId(List<Nip01Event> events, String id) {
  return events.singleWhere((event) {
    return event.tags.where((tag) => tag.firstOrNull == 'imeta').any((tag) {
      return tag.skip(1).where((field) => field.startsWith('url ')).any((
        field,
      ) {
        final uri = Uri.tryParse(field.substring('url '.length));
        return uri?.path.endsWith('/$id.mp4') == true;
      });
    });
  });
}
