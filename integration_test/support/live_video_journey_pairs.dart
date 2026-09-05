part of 'live_video_journey.dart';

extension LiveVideoJourneyPairs on LiveVideoJourney {
  Future<void> comparePair(VideoPost post, {required bool directFirst}) async {
    final url = Uri.parse(post.media.remoteUrl!);
    log.add('paired_control', {
      'eventId': post.id.value,
      'directFirst': directFirst,
      'url': '$url',
    });
    if (directFirst) await liveDirectPlayback(tester, log, url);
    await _isolatedWarp(post);
    if (!directFirst) await liveDirectPlayback(tester, log, url);
    await liveOriginProbe(log, url);
  }

  Future<void> _isolatedWarp(VideoPost post) async {
    await tester.pumpWidget(const SizedBox.shrink());
    runtime.focus.focusChanged(FeedFocus.around(posts: [post], activeIndex: 0));
    final surface = runtime.dependencies!.videoPlaybackPort.buildSurface(
      VideoPlaybackSurfaceRequest(
        media: post.media,
        videoId: PlaybackVideoId.parse(post.id.value),
        playbackDeliveryId: post.media.playbackDeliveryId,
        isActive: true,
      ),
    );
    await tester.pumpWidget(
      LiveComparisonSurface(
        label: 'WARP replay',
        host: Uri.parse(post.media.remoteUrl!).host,
        child: surface,
      ),
    );
    await sample(currentFocus!, 'pinned_warp');
    await captureEvidence();
    await tester.pumpWidget(const SizedBox.shrink());
    runtime.focus.clearFocus();
    await pumpFor(const Duration(seconds: 1));
  }
}
