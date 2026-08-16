import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_screen.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_video_interaction.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/nostr_reference.dart';
import '../support/sample_data.dart';
import '../support/scripted_feed_repository.dart';

void main() {
  testWidgets('a safe revision preserves the viewer playback mode', (
    tester,
  ) async {
    final initial = samplePost(
      id: 'initial-event',
      nostrReference: _coordinate(testEventId),
    );
    final revised = samplePost(
      id: 'revised-event',
      nostrReference: _coordinate(secondTestEventId),
    ).withMedia(initial.media);
    final source = ScriptedFeedRepository(
      loads: [
        [initial],
        [revised],
      ],
    );
    final engagement = FakeVideoCatalogRepository(forYouFeed: const []);
    final playback = FakeVideoPlaybackPort();
    await tester.pumpWidget(
      feedScreenHarness(
        engagement,
        options: FeedScreenHarnessOptions(feed: source, playbackPort: playback),
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.byType(FeedVideoInteraction));
    await tester.pump();
    expect(find.byIcon(Icons.play_arrow_rounded), findsOneWidget);

    final cubit = tester.element(find.byType(FeedScreen)).read<FeedCubit>();
    await cubit.refresh();
    await tester.pump();

    expect(find.byIcon(Icons.play_arrow_rounded), findsOneWidget);
    expect(
      playback.requests.map((request) => request.videoId).toSet(),
      hasLength(1),
    );
  });
}

NostrEventReference _coordinate(String eventId) => nostrReference(
  eventId: eventId,
  kind: 34236,
  identifier: 'stable-coordinate',
);
