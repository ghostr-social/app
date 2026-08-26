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
  test('terminal failure only overrides its exact prepared authority', () {
    final post = samplePost(id: 'intended');
    final asset = readyPlaybackPreparation(post.media);
    final preparation = FeedPlaybackPreparation.managed(
      revision: BigInt.one,
      current: asset.bind(post.media),
    );
    final matching = _evidence(post, preparation, asset.authority);
    final stale = _evidence(
      post,
      preparation,
      PlaybackAssetAuthority(
        deliveryId: asset.deliveryId,
        representationId: asset.representationId,
        assetId: PlaybackAssetId.parse(
          'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
        ),
      ),
    );

    expect(matching.isReadyAt(0), isFalse);
    expect(stale.isReadyAt(0), isTrue);
  });
}

FeedReadinessEvidence _evidence(
  VideoPost post,
  FeedPlaybackPreparation preparation,
  PlaybackAssetAuthority authority,
) => FeedReadinessEvidence(
  posts: [post],
  delivery: {
    post.media.playbackDeliveryId!: VideoDeliverySnapshot(
      deliveryId: post.media.playbackDeliveryId!,
      phase: VideoDeliveryPhase.failed,
      bytesPresent: BigInt.zero,
      authority: authority,
    ),
  },
  preparation: preparation,
);
