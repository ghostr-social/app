part of 'live_video_journey.dart';

extension LiveVideoJourneyControls on LiveVideoJourney {
  Future<void> controls() async {
    await tester.pumpWidget(const SizedBox.shrink());
    runtime.focus.clearFocus();
    await pumpFor(const Duration(seconds: 1));
    final urls = <String>{};
    final ids = samples.map((sample) => sample['eventId']! as String).toSet();
    final replies = await liveRelayRead(log, runtime.environment.relays, ids);
    log.add('live_corpus', {
      'observedEventIds': ids.toList(),
      'refetchedEventIds': replies.keys.toList(),
    });
    var successful = 0;
    for (final sample in samples) {
      final url = sample['url'];
      if (url is! String) continue;
      final healthy = _healthy(sample);
      if (healthy && successful++ >= 5) continue;
      urls.add(url);
    }
    for (final url in urls) {
      await liveDirectPlayback(tester, log, Uri.parse(url));
      await liveOriginProbe(log, Uri.parse(url));
    }
  }
}

bool _healthy(Map<String, Object?> sample) =>
    sample['renderedAndMoving'] == true &&
    sample['unavailableVisible'] == false &&
    (sample['firstFrameMs'] as int) < (sample['targetMs'] as int) &&
    (sample['longestFreezeMs'] as int? ?? 0) <= 2000;
