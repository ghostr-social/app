import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_ready_selector.dart';

import '../support/sample_data.dart';

void main() {
  test('only the current HLS asset frame can settle or drive rescue', () {
    final posts = List.generate(3, _post);
    final intended = _authority(posts[1].media, 4);
    final neighbor = _authority(posts[2].media, 2);
    final delivery = {
      intended.deliveryId: _snapshot(intended, VideoDeliveryPhase.preparing),
      neighbor.deliveryId: _snapshot(neighbor, VideoDeliveryPhase.startable),
    };

    final settled = _decision(posts, delivery, {intended});
    final rescued = _decision(posts, delivery, {neighbor});

    expect(settled.reason, FeedReadyReason.intendedReady);
    expect(rescued.action, FeedReadyAction.rescue);
    expect(rescued.selectedIndex, 2);

    final stale = _authority(posts[2].media, 1);
    final wrongDelivery = _authority(
      posts[2].media,
      2,
      delivery: 'different-post',
    );
    for (final rejected in [stale, wrongDelivery]) {
      final evidence = _evidence(posts, delivery, {rejected});
      final decision = const FeedReadySelector().select(
        evidence,
        fromIndex: 0,
        intendedIndex: 1,
      );
      expect(evidence.isPlayerVerifiedAt(2), isFalse);
      expect(decision.action, FeedReadyAction.intended);
    }
  });
}

FeedReadyDecision _decision(
  List<VideoPost> posts,
  Map<PlaybackDeliveryId, VideoDeliverySnapshot> delivery,
  Set<HlsPlaybackAuthority> verified,
) {
  return const FeedReadySelector().select(
    _evidence(posts, delivery, verified),
    fromIndex: 0,
    intendedIndex: 1,
  );
}

FeedReadinessEvidence _evidence(
  List<VideoPost> posts,
  Map<PlaybackDeliveryId, VideoDeliverySnapshot> delivery,
  Set<HlsPlaybackAuthority> verified,
) => FeedReadinessEvidence(
  posts: posts,
  delivery: delivery,
  verifiedHlsAuthorities: verified,
);

VideoPost _post(int index) => samplePost(id: 'p$index').withMedia(
  VideoMediaSource.withCacheScope(
    VideoMediaSource.remote(
      'https://media.test/$index.m3u8',
      delivery: VideoMediaDelivery.hls,
    ),
    'p$index',
  ),
);

VideoDeliverySnapshot _snapshot(
  HlsPlaybackAuthority authority,
  VideoDeliveryPhase phase,
) => VideoDeliverySnapshot(
  deliveryId: authority.deliveryId,
  phase: phase,
  bytesPresent: BigInt.zero,
  hlsAuthority: authority,
);

HlsPlaybackAuthority _authority(
  VideoMediaSource media,
  int revision, {
  String? delivery,
}) => HlsPlaybackAuthority(
  deliveryId: PlaybackDeliveryId.parse(delivery ?? media.cacheScope!.value),
  representationId: VideoRepresentationId.forMedia(media),
  assetRevision: HlsPlaybackAssetRevision.parse(BigInt.from(revision)),
);
