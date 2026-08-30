import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/controlled_video_delivery_updates.dart';
import '../support/fake_media_ports.dart';
import '../support/fake_video_catalog_repository.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('projects only the exact prepared HLS authority into playback', (
    tester,
  ) async {
    final post = samplePost(id: 'hls').withMedia(_media());
    final repository = FakeVideoCatalogRepository(forYouFeed: [post]);
    final delivery = ControlledVideoDeliveryUpdates();
    final playback = FakeVideoPlaybackPort();
    addTearDown(delivery.close);
    await tester.pumpWidget(
      feedScreenHarness(
        repository,
        options: FeedScreenHarnessOptions(
          deliveryUpdates: delivery,
          playbackPort: playback,
        ),
      ),
    );
    await tester.pumpAndSettle();
    final exact = _authority(post.media, 'a', 1);

    delivery.publish(
      post,
      phase: VideoDeliveryPhase.startable,
      hlsAuthority: exact,
    );
    await tester.pumpAndSettle();

    expect(_loaded(tester).hlsAuthorityFor(post.media), exact);
    final request = playback.requests.last;
    expect(request.hlsAuthority, exact);
    request.onHlsFirstFrameRendered?.call(exact);
    await tester.pumpAndSettle();
    expect(_loaded(tester).isHlsPlayerVerified(exact), isTrue);
    request.onHlsDecodedReadinessRevoked?.call(exact);
    await tester.pump();
    expect(_loaded(tester).isHlsPlayerVerified(exact), isFalse);

    delivery.publish(
      post,
      phase: VideoDeliveryPhase.startable,
      hlsAuthority: _authority(post.media, 'b', 2),
    );
    await tester.pump();

    expect(_loaded(tester).hlsAuthorityFor(post.media), isNull);
    expect(playback.requests.last.hlsAuthority, isNull);
  });
}

FeedLoaded _loaded(WidgetTester tester) {
  final context = tester.element(find.byType(PageView));
  return BlocProvider.of<FeedCubit>(context).state as FeedLoaded;
}

VideoMediaSource _media() => VideoMediaSource.withCacheScope(
  VideoMediaSource.remote(
    'https://media.test/root.m3u8',
    delivery: VideoMediaDelivery.hls,
  ),
  'hls',
);

HlsPlaybackAuthority _authority(
  VideoMediaSource media,
  String fingerprint,
  int revision,
) {
  return HlsPlaybackAuthority(
    deliveryId: media.playbackDeliveryId!,
    representationId: fingerprint == 'a'
        ? VideoRepresentationId.forMedia(media)
        : VideoRepresentationId.parse(fingerprint * 64),
    assetRevision: HlsPlaybackAssetRevision.parse(BigInt.from(revision)),
  );
}
