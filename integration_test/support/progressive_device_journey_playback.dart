part of 'progressive_device_journey.dart';

extension ProgressiveDeviceJourneyPlayback on ProgressiveDeviceJourney {
  Future<void> waitForPreparedNext(WidgetTester tester) {
    return waitForDeviceCondition(
      tester,
      () => origin.rangesFor('next').isNotEmpty,
    );
  }

  Future<void> showCurrent(WidgetTester tester) async {
    _currentFocus = await _show(tester, posts[0]);
  }

  Future<void> showNext(WidgetTester tester) async {
    _focus.focusChanged(FeedFocus.around(posts: posts, activeIndex: 1));
    _nextFocus = await _show(tester, posts[1]);
  }

  Future<void> waitForCurrentFrame(WidgetTester tester) {
    return _waitForFrame(tester, _currentFocus!);
  }

  Future<void> waitForNextFrame(WidgetTester tester) {
    return _waitForFrame(tester, _nextFocus!);
  }

  Future<void> waitForAcceptedPlayback(WidgetTester tester, int minimum) {
    return waitForAsyncDeviceCondition(tester, () async {
      final admissions = await _admissions.delta();
      return admissions.accepted >= BigInt.from(minimum);
    });
  }

  Future<void> waitForAcceptedDelivery(WidgetTester tester, String expected) {
    return waitForAsyncDeviceCondition(tester, () async {
      final admissions = await _admissions.delta();
      return admissions.lastAcceptedDeliveryId == expected;
    });
  }

  Future<void> pumpFor(WidgetTester tester, Duration duration) {
    return pumpDeviceFor(tester, duration);
  }

  bool hasPlaybackError(WidgetTester tester) {
    return find.text('Video unavailable').evaluate().isNotEmpty;
  }

  Future<PlaybackFocus> _show(WidgetTester tester, VideoPost post) async {
    final id = PlaybackVideoId.parse(post.id.value);
    final focus = _telemetry.probe.markFocus(id);
    await tester.pumpWidget(
      MaterialApp(
        home: _playback.buildSurface(
          VideoPlaybackSurfaceRequest(
            media: post.media,
            videoId: id,
            isActive: true,
          ),
        ),
      ),
    );
    return focus;
  }

  Future<void> _waitForFrame(WidgetTester tester, PlaybackFocus focus) {
    return waitForDeviceCondition(
      tester,
      () => _telemetry.probe.playingLatency(focus) != null,
    );
  }
}
