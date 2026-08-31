import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';

import '../../integration_test/support/warp_feed_playback_journey.dart';
import 'warp_replay_store_history_fixture.dart';

void main() {
  test('accepts only bounded authoritative native store history', () {
    final deliveryId = PlaybackDeliveryId.parse('delivery');
    final authority = replayStoreAuthority(deliveryId, 'a');
    final otherAuthority = replayStoreAuthority(deliveryId, 'b');
    final valid = [
      replayStoreSnapshot(deliveryId, bytes: 0),
      replayStoreSnapshot(
        deliveryId,
        bytes: 64,
        total: 100,
        authority: authority,
      ),
    ];

    expect(
      warpNativeStoreHistoryIsValid(valid, deliveryId, BigInt.from(100)),
      isTrue,
    );
    expect(
      warpNativeStoreHistoryIsValid(
        [
          replayStoreSnapshot(
            deliveryId,
            bytes: 101,
            total: 100,
            authority: authority,
          ),
        ],
        deliveryId,
        BigInt.from(100),
      ),
      isFalse,
    );
    expect(
      warpNativeStoreHistoryIsValid(
        [replayStoreFailedSnapshot(deliveryId, 100)],
        deliveryId,
        BigInt.from(100),
      ),
      isFalse,
    );
    expect(
      warpNativeStoreHistoryIsValid(
        [replayStoreSnapshot(deliveryId, bytes: 64, total: 100)],
        deliveryId,
        BigInt.from(100),
      ),
      isFalse,
    );
    expect(
      warpNativeStoreHistoryIsValid(
        [
          replayStoreSnapshot(deliveryId, bytes: 32, total: 100),
          replayStoreSnapshot(
            deliveryId,
            bytes: 64,
            total: 100,
            authority: authority,
          ),
        ],
        deliveryId,
        BigInt.from(100),
      ),
      isFalse,
    );
    expect(
      warpNativeStoreHistoryIsValid(
        [
          replayStoreSnapshot(
            deliveryId,
            bytes: 32,
            total: 100,
            authority: authority,
          ),
          replayStoreSnapshot(
            deliveryId,
            bytes: 64,
            total: 100,
            authority: otherAuthority,
          ),
        ],
        deliveryId,
        BigInt.from(100),
      ),
      isFalse,
    );
  });
}
