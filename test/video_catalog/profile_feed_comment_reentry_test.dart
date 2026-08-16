import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/comments/presentation/comments_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/profile_video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_feed_screen.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/fake_video_sharing.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('profile comments preserve the current feed session', (
    tester,
  ) async {
    final creator = sampleCreator();
    final post = samplePost(creator: creator, caption: 'Profile clip');
    final history = FakeWatchHistoryRepository();
    final repository = FakeVideoCatalogRepository(
      forYouFeed: const [],
      feed: FakeFeedScenario(
        profiles: {
          creator.id: sampleProfileDetails(profile: creator, posts: [post]),
        },
      ),
    );
    await tester.pumpWidget(
      MaterialApp(
        home: BlocProvider(
          create: (_) => FeedCubit(
            FeedDependencies(
              feed: ProfileVideoFeedRepository(
                profile: repository,
                viewer: sampleCreator(id: 'viewer'),
                creatorId: creator.id,
              ),
              engagement: repository,
              optional: FeedOptionalDependencies(
                watch: FeedWatchDependencies(
                  tracker: WatchHistoryTracker(
                    history: history,
                    failureReporter: RecordingFailureReporter(),
                  ),
                ),
              ),
            ),
          )..load(),
          child: ProfileFeedScreen(
            request: ProfileFeedRequest(
              creator: creator,
              playbackPort: FakeVideoPlaybackPort(),
              shareWorkflow: FakeVideoShareWorkflow(),
              createComments: (value) => CommentsCubit(repository, value),
              onOpenProfile: (_) async {},
              onOpenHashtag: (_) async {},
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    expect(history.entries, hasLength(1));

    await tester.tap(find.byTooltip('Open comments'));
    await tester.pumpAndSettle();
    await tester.tapAt(const Offset(10, 10));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 500));

    expect(find.text('Profile clip'), findsOneWidget);
    expect(history.entries, hasLength(1));
    expect(history.entries.single.videoId, 'e:post-1');
  });
}
