import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_feed_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/nostr_test_values.dart';
import '../support/sample_data.dart';
import '../support/scripted_feed_repository.dart';

void main() {
  test(
    'an active revision cannot switch to previously watched media',
    () async {
      final watched = samplePost(id: 'watched-media');
      final initial = samplePost(
        id: 'initial-event',
        nostrReference: _addressable(testEventId),
      );
      final revision = samplePost(
        id: 'revision-event',
        nostrReference: _addressable(secondTestEventId),
      ).withMedia(watched.media);
      final history = FakeWatchHistoryRepository(
        entries: [
          WatchHistoryEntry.fromPost(watched, DateTime.utc(2026, 8, 15)),
        ],
      );
      final source = ScriptedFeedRepository(
        loads: [
          [initial],
          [revision],
        ],
      );
      final reporter = RecordingFailureReporter();
      final feed = WatchAwareVideoFeedRepository(
        feed: source,
        history: history,
        failureReporter: reporter,
      );
      final cubit = FeedCubit(
        FeedDependencies(
          feed: feed,
          engagement: FakeVideoCatalogRepository(forYouFeed: const []),
          optional: FeedOptionalDependencies(
            watch: FeedWatchDependencies(
              tracker: WatchHistoryTracker(
                history: history,
                failureReporter: reporter,
              ),
            ),
          ),
        ),
      );
      addTearDown(cubit.close);
      await cubit.load();

      await cubit.refresh();

      final active = (cubit.state as FeedLoaded).roster.active;
      expect(active.id, initial.id);
      expect(active.media.remoteUrl, initial.media.remoteUrl);
    },
  );
}

NostrEventReference _addressable(String eventId) {
  return NostrEventReference(
    eventId: NostrEventId.parse(eventId),
    authorPublicKeyHex: NostrPublicKeyHex.parse(testAuthorPublicKey),
    kind: NostrEventKind.parse(34236),
    details: NostrEventReferenceDetails(
      identifier: NostrEventIdentifier.parse('stable-coordinate'),
    ),
  );
}
