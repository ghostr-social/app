import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_asset_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_preparation_reducer.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_ready_selector.dart';

import '../support/ready_playback_preparation.dart';
import '../support/sample_data.dart';

void main() {
  test('renewed delivery authority cannot consume an old player frame', () {
    final posts = List.generate(3, (index) => samplePost(id: 'p$index'));
    final oldReady = readyPlaybackPreparation(posts[2].media);
    final evidence = FeedReadinessEvidence(
      posts: posts,
      delivery: {
        posts[1].media.playbackDeliveryId!: _snapshot(posts, 1),
        posts[2].media.playbackDeliveryId!: _snapshot(
          posts,
          2,
          authority: _renewed(oldReady.authority),
        ),
      },
      preparation: FeedPlaybackPreparation.managed(
        revision: BigInt.one,
        upcoming: [oldReady.bind(posts[2].media)],
      ),
    );

    final decision = const FeedReadySelector().select(
      evidence,
      fromIndex: 0,
      intendedIndex: 1,
    );

    expect(evidence.isPlayerVerifiedAt(2), isFalse);
    expect(decision.action, FeedReadyAction.intended);
    expect(decision.reason, FeedReadyReason.noReadyAlternative);
  });
}

VideoDeliverySnapshot _snapshot(
  List<VideoPost> posts,
  int index, {
  PlaybackAssetAuthority? authority,
}) {
  return VideoDeliverySnapshot(
    deliveryId: posts[index].media.playbackDeliveryId!,
    phase: index == 1
        ? VideoDeliveryPhase.preparing
        : VideoDeliveryPhase.startable,
    bytesPresent: BigInt.zero,
    authority: authority,
  );
}

PlaybackAssetAuthority _renewed(PlaybackAssetAuthority old) {
  return PlaybackAssetAuthority(
    deliveryId: old.deliveryId,
    representationId: old.representationId,
    assetId: PlaybackAssetId.parse(
      'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
    ),
  );
}
