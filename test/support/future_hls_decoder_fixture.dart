import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

import 'controlled_video_delivery_updates.dart';
import 'fake_media_ports.dart';
import 'fake_video_catalog_repository.dart';
import 'feed_screen_harness.dart';
import 'sample_data.dart';

final class FutureHlsDecoderFixture {
  final posts = [samplePost(id: 'p0'), ...List.generate(5, _hlsPost)];
  final delivery = ControlledVideoDeliveryUpdates();
  final playback = FakeVideoPlaybackPort();

  Future<void> mountAndPublish(WidgetTester tester) async {
    await tester.pumpWidget(
      feedScreenHarness(
        FakeVideoCatalogRepository(forYouFeed: posts),
        options: FeedScreenHarnessOptions(
          deliveryUpdates: delivery,
          playbackPort: playback,
        ),
      ),
    );
    await tester.pumpAndSettle();
    for (final post in posts.skip(1)) {
      delivery.publish(
        post,
        phase: VideoDeliveryPhase.startable,
        hlsAuthority: _authority(post),
      );
    }
    await tester.pumpAndSettle();
  }

  VideoPlaybackSurfaceRequest request(String id) {
    return playback.requests.lastWhere(
      (request) => request.videoId?.value == id,
    );
  }

  HlsPlaybackAuthority authority(String id) {
    return _authority(posts.singleWhere((post) => post.id.value == id));
  }

  void render(String id) {
    final prepared = request(id);
    prepared.onHlsFirstFrameRendered?.call(prepared.hlsAuthority!);
  }

  Future<void> promoteNextAndReturn(WidgetTester tester) async {
    final cubit = tester.element(find.byType(PageView)).read<FeedCubit>();
    cubit.pageChanged(1);
    await tester.pumpAndSettle();
    render('h1');
    cubit.pageChanged(0);
    await tester.pumpAndSettle();
  }

  Future<void> close() => delivery.close();
}

VideoPost _hlsPost(int index) {
  final id = 'h$index';
  final media = VideoMediaSource.withCacheScope(
    VideoMediaSource.remote(
      'https://media.test/$id/master.m3u8',
      delivery: VideoMediaDelivery.hls,
    ),
    id,
  );
  return samplePost(id: id).withMedia(media);
}

HlsPlaybackAuthority _authority(VideoPost post) => HlsPlaybackAuthority(
  deliveryId: post.media.playbackDeliveryId!,
  representationId: VideoRepresentationId.forMedia(post.media),
  assetRevision: HlsPlaybackAssetRevision.parse(BigInt.one),
);
