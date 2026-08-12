import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/comments/presentation/comments_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/query_video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/presentation/discovery_feed_screen.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_screen.dart';

import '../support/fakes.dart';
import '../support/fake_video_sharing.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('a hashtag feed plays search matches under its own title', (
    tester,
  ) async {
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [samplePost(caption: 'Tagged clip')],
    );
    await tester.pumpWidget(
      MaterialApp(
        home: BlocProvider(
          create: (_) => FeedCubit(
            FeedDependencies(
              feed: QueryVideoFeedRepository(
                search: repository,
                query: '#dance',
              ),
              engagement: repository,
              optional: FeedOptionalDependencies(social: repository),
            ),
          )..load(),
          child: DiscoveryFeedScreen(
            request: DiscoveryFeedRequest(
              query: '#dance',
              playbackPort: FakeVideoPlaybackPort(),
              shareWorkflow: FakeVideoShareWorkflow(),
              createComments: (post) => CommentsCubit(repository, post),
              onOpenProfile: (_) async {},
              onOpenHashtag: (_) async {},
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.widgetWithText(AppBar, '#dance'), findsOneWidget);
    expect(find.byType(FeedScreen), findsOneWidget);
    expect(find.text('Tagged clip'), findsOneWidget);
    // The feed also tops itself up once, so the query repeats.
    expect(repository.searchQueries, everyElement('#dance'));
    expect(repository.searchQueries, isNotEmpty);
  });
}
