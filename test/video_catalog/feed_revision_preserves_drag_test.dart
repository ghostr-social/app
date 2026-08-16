import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_screen.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_page_view.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/nostr_reference.dart';
import '../support/sample_data.dart';
import '../support/scripted_feed_repository.dart';

void main() {
  testWidgets('a safe first-row revision preserves an in-progress drag', (
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
    final second = samplePost(id: 'second');
    final source = ScriptedFeedRepository(
      loads: [
        [initial, second],
        [revised, second],
      ],
    );
    final engagement = FakeVideoCatalogRepository(forYouFeed: const []);
    await tester.pumpWidget(
      feedScreenHarness(
        engagement,
        options: FeedScreenHarnessOptions(feed: source),
      ),
    );
    await tester.pumpAndSettle();
    final gesture = await tester.startGesture(
      tester.getCenter(find.byType(FeedPageView)),
    );
    await gesture.moveBy(const Offset(0, -300));
    await tester.pump();
    final before = tester
        .state<ScrollableState>(find.byType(Scrollable))
        .position
        .pixels;

    final cubit = tester.element(find.byType(FeedScreen)).read<FeedCubit>();
    await cubit.refresh();
    await tester.pump();

    final after = tester
        .state<ScrollableState>(find.byType(Scrollable))
        .position
        .pixels;
    expect(after, closeTo(before, 1));
    await gesture.cancel();
  });
}

NostrEventReference _coordinate(String eventId) => nostrReference(
  eventId: eventId,
  kind: 34236,
  identifier: 'stable-coordinate',
);
