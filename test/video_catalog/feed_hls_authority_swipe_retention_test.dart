import 'dart:async';

import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/controlled_video_delivery_updates.dart';
import '../support/fake_video_catalog_repository.dart';
import '../support/sample_data.dart';

void main() {
  test('retains exact HLS authority and decoded readiness across a swipe', () {
    fakeAsync((clock) {
      final media = VideoMediaSource.withCacheScope(
        VideoMediaSource.remote(
          'https://media.test/swipe.m3u8',
          delivery: VideoMediaDelivery.hls,
        ),
        'swipe-hls',
      );
      final hls = samplePost(id: 'hls').withMedia(media);
      final posts = [samplePost(id: 'first'), hls];
      final updates = ControlledVideoDeliveryUpdates();
      final repository = FakeVideoCatalogRepository(forYouFeed: posts);
      final cubit = FeedCubit(
        FeedDependencies(
          feed: repository,
          engagement: repository,
          optional: FeedOptionalDependencies(
            delivery: FeedDeliveryDependencies(deliveryUpdates: updates),
          ),
        ),
      );
      final authority = HlsPlaybackAuthority(
        deliveryId: media.playbackDeliveryId!,
        representationId: VideoRepresentationId.forMedia(media),
        assetRevision: HlsPlaybackAssetRevision.parse(BigInt.one),
      );

      unawaited(cubit.load());
      updates.publish(
        hls,
        phase: VideoDeliveryPhase.startable,
        hlsAuthority: authority,
      );
      clock.flushMicrotasks();
      cubit.hlsFirstFrameRendered(authority);
      cubit.pageChanged(1);
      clock.flushMicrotasks();

      final moved = cubit.state as FeedLoaded;
      expect(moved.activeIndex, 1);
      expect(moved.hlsAuthorityFor(media), authority);
      expect(moved.isHlsPlayerVerified(authority), isTrue);
      unawaited(cubit.close());
      unawaited(updates.close());
      clock.flushMicrotasks();
    });
  });
}
