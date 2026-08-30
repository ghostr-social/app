import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_preparation_reducer.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_ready_selector.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';

import '../support/ready_playback_preparation.dart';
import '../support/sample_data.dart';

void main() {
  test('backward projection retains the centered Ready asset', () {
    final posts = List.generate(4, (index) => samplePost(id: 'p$index'));
    final p2 = readyPlaybackPreparation(posts[2].media);
    final p3 = readyPlaybackPreparation(posts[3].media);
    final reducer = FeedPreparationReducer();
    final centered = reducer.acceptWindow(
      PlaybackPreparationPlan(
        revision: BigInt.one,
        currentDeliveryId: p2.deliveryId,
        current: p2,
        upcoming: [p3],
      ),
      posts[2].media,
      [posts[3].media],
    );

    final projected = reducer.realignWindow(centered!, posts[1].media, [
      posts[2].media,
      posts[3].media,
    ]);
    final decision = const FeedReadySelector().select(
      FeedReadinessEvidence(
        posts: posts,
        delivery: {
          p2.deliveryId: VideoDeliverySnapshot(
            deliveryId: p2.deliveryId,
            phase: VideoDeliveryPhase.preparing,
            bytesPresent: BigInt.zero,
          ),
        },
        preparation: projected,
      ),
      fromIndex: 1,
      intendedIndex: 2,
    );

    expect(
      (
        nextIsP2: projected.next?.matches(posts[2].media) == true,
        authority: projected.next?.authority,
        ready: projected.next?.readiness.isPlayerVerified == true,
        action: decision.action,
        selectedIndex: decision.selectedIndex,
      ),
      (
        nextIsP2: true,
        authority: p2.authority,
        ready: true,
        action: FeedReadyAction.intended,
        selectedIndex: 2,
      ),
    );
  });
}
