part of 'live_video_journey.dart';

extension LiveVideoJourneyPins on LiveVideoJourney {
  Future<void> replayPins() async {
    final ids = const String.fromEnvironment(
      'LIVE_VIDEO_EVENT_IDS',
    ).split(',').toSet();
    await tester.pumpWidget(const SizedBox.shrink());
    runtime.focus.clearFocus();
    final replies = await liveRelayRead(log, runtime.environment.relays, ids);
    var index = 0;
    for (final id in ids) {
      final event = replies[id];
      if (event == null) {
        failures.add(
          'Pinned event $id was not returned by any configured relay.',
        );
        continue;
      }
      final post = await _pinnedPost(event);
      if (post == null) {
        failures.add('Production feed could not map pinned event $id.');
        continue;
      }
      await comparePair(post, directFirst: index.isOdd);
      index++;
    }
  }

  Future<VideoPost?> _pinnedPost(Nip01Event event) async {
    final known = runtime.focus.posts[event.id];
    if (known != null) return known;
    final source = runtime.environment.delivery!.remoteSource;
    try {
      final posts = await source.loadRemoteFeed(
        creatorIds: {ProfileId.parse(Nip19.encodePubKey(event.pubKey))},
        olderThan: DateTime.fromMillisecondsSinceEpoch(
          (event.createdAt + 1) * 1000,
          isUtc: true,
        ),
      );
      return posts.where((post) => post.id.value == event.id).firstOrNull;
    } on Object catch (error) {
      log.add('pin_mapping_failed', {'eventId': event.id, 'error': '$error'});
      return null;
    }
  }
}
