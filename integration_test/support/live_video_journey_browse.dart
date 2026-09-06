part of 'live_video_journey.dart';

extension LiveVideoJourneyBrowse on LiveVideoJourney {
  Future<void> browse() async {
    const count = int.fromEnvironment('LIVE_VIDEO_COUNT', defaultValue: 20);
    for (
      var index = 0;
      index < count + 500 && visited.length < count;
      index++
    ) {
      await waitUntil(() => currentFocus != null);
      final focus = currentFocus;
      if (focus == null) {
        failures.add('No focused video at sample $index.');
        break;
      }
      final id = focus.videoId.value;
      final url = runtime.focus.posts[id]?.media.remoteUrl;
      if (corpus.admit(id, url)) {
        visited.add(id);
        await sample(focus, index == 0 ? 'startup' : 'browse');
        await captureEvidence();
      } else {
        log.add('corpus_excluded_video', {'eventId': id, 'url': url});
      }
      if (visited.length < count && !await swipe(LiveSwipeDirection.forward)) {
        failures.add('Feed did not advance after sample $index.');
        break;
      }
    }
    if (visited.length < count) {
      failures.add('Collected only ${visited.length} of $count fresh videos.');
    }
    log.add('fresh_corpus', {
      'sampled': visited.length,
      'requested': count,
      'hosts': corpus.hosts,
    });
  }

  Future<void> warmReturn() async {
    for (var index = 0; index < 5 && index < visited.length - 1; index++) {
      if (!await swipe(LiveSwipeDirection.backward)) {
        failures.add('Warm return could not reach prior video $index.');
        return;
      }
      final focus = currentFocus;
      if (focus != null) await sample(focus, 'warm_return');
    }
  }

  Future<void> rapidSwipes() async {
    final page = find.byType(PageView).first;
    if (page.evaluate().isEmpty) return;
    final gesture = LiveVideoSwipe(tester, log);
    for (var index = 0; index < 10; index++) {
      await gesture.perform(LiveSwipeDirection.forward, currentFocus);
      await pumpFor(deviceRapidSwipeCadence - deviceRapidSwipeGestureTarget);
    }
    await pumpFor(const Duration(milliseconds: 500));
    final focus = currentFocus;
    if (focus != null) await sample(focus, 'rapid_final');
  }
}
