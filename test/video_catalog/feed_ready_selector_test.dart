import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_ready_selector.dart';

import '../support/player_verified_preparation.dart';
import '../support/sample_data.dart';

void main() {
  test('selects the nearest ready candidate inside the bounded window', () {
    final posts = List.generate(5, (index) => samplePost(id: 'p$index'));
    final delivery = {
      posts[1].media.playbackDeliveryId!: snapshot(posts, 1, startable: false),
      posts[2].media.playbackDeliveryId!: snapshot(posts, 2, startable: true),
      posts[4].media.playbackDeliveryId!: snapshot(posts, 4, startable: true),
    };

    final decision = const FeedReadySelector().select(
      FeedReadinessEvidence(
        posts: posts,
        delivery: delivery,
        preparation: playerVerifiedWindow(
          posts,
          currentIndex: 0,
          readyIndices: [2, 4],
        ),
      ),
      fromIndex: 0,
      intendedIndex: 1,
    );

    expect(decision.action, FeedReadyAction.rescue);
    expect(decision.selectedIndex, 2);
    expect(decision.displacement, 1);
  });
}

VideoDeliverySnapshot snapshot(
  List<VideoPost> posts,
  int index, {
  required bool startable,
}) {
  return VideoDeliverySnapshot(
    deliveryId: posts[index].media.playbackDeliveryId!,
    phase: startable
        ? VideoDeliveryPhase.startable
        : VideoDeliveryPhase.preparing,
    bytesPresent: BigInt.zero,
  );
}
