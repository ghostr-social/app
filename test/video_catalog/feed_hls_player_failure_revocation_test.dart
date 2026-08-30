import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:ghostr/platform/media/hls_video_playback_port.dart';
import 'package:ghostr/platform/media/native_rendered_first_frame_port.dart';
import 'package:ghostr/platform/media/rendered_first_frame_protocol.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/controlled_video_delivery_updates.dart';
import '../support/fake_hls_playback_gateway.dart';
import '../support/fake_video_catalog_repository.dart';
import '../support/feed_screen_harness.dart';
import '../support/recovering_video_player_platform.dart';
import '../support/sample_data.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('controller failure revokes decoded HLS readiness immediately', (
    tester,
  ) async {
    final media = VideoMediaSource.withCacheScope(
      VideoMediaSource.remote(
        'https://media.test/recovery.m3u8',
        delivery: VideoMediaDelivery.hls,
      ),
      'hls-recovery',
    );
    final post = samplePost(id: 'hls').withMedia(media);
    final updates = ControlledVideoDeliveryUpdates();
    final authority = HlsPlaybackAuthority(
      deliveryId: media.playbackDeliveryId!,
      representationId: VideoRepresentationId.forMedia(media),
      assetRevision: HlsPlaybackAssetRevision.parse(BigInt.one),
    );
    final nativeEvents = StreamController<Object?>();
    final frames = NativeRenderedFirstFramePort(events: nativeEvents.stream);
    final platform = RecoveringVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final gateway = FakeHlsPlaybackGateway();
    final playback = HlsVideoPlaybackPort(
      gateway: gateway,
      delegate: VideoPlayerPlaybackPort(
        recoveryPolicy: PlaybackRecoveryPolicy([const Duration(seconds: 1)]),
        renderedFirstFrames: frames,
      ),
    );
    addTearDown(updates.close);
    addTearDown(() async {
      await frames.dispose();
      await nativeEvents.close();
    });

    await tester.pumpWidget(
      feedScreenHarness(
        FakeVideoCatalogRepository(forYouFeed: [post]),
        options: FeedScreenHarnessOptions(
          deliveryUpdates: updates,
          playbackPort: playback,
        ),
      ),
    );
    updates.publish(
      post,
      phase: VideoDeliveryPhase.startable,
      hlsAuthority: authority,
    );
    await tester.pump();
    expect(gateway.requests.single.expectedAuthority, authority);
    gateway.completeNext();
    await tester.pump(const Duration(milliseconds: 100));
    await settleVideoPlayerTasks(tester);
    final token =
        platform.dataSources.single.httpHeaders[warpPlaybackAttemptHeader];
    nativeEvents.add({'version': 1, 'attemptToken': token});
    await settleVideoPlayerTasks(tester);
    final cubit = tester.element(find.byType(PageView)).read<FeedCubit>();
    expect((cubit.state as FeedLoaded).isHlsPlayerVerified(authority), isTrue);

    platform.failLatest('origin reset');
    await settleVideoPlayerTasks(tester);

    expect(gateway.activeLeaseCount, 1);
    expect(platform.commands, contains('dispose:0'));
    expect((cubit.state as FeedLoaded).isHlsPlayerVerified(authority), isFalse);
  });
}
