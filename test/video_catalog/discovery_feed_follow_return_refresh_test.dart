import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/comments/presentation/comments_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/query_video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/presentation/discovery_feed_screen.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/fake_video_sharing.dart';
import '../support/follow_profile_workflow.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('returning from a discovery profile refreshes follows', (
    tester,
  ) async {
    final creator = sampleCreator();
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [samplePost(creator: creator)],
    );
    final cubit = FeedCubit(
      FeedDependencies(
        viewerId: sampleSession().profile.id,
        feed: QueryVideoFeedRepository(search: repository, query: '#dance'),
        engagement: repository,
        followProfile: testFollowProfileWorkflow(repository),
        optional: FeedOptionalDependencies(social: repository),
      ),
    );
    addTearDown(cubit.close);
    final request = DiscoveryFeedRequest(
      query: '#dance',
      playbackPort: FakeVideoPlaybackPort(),
      shareWorkflow: FakeVideoShareWorkflow(),
      createComments: (post) => CommentsCubit(repository, post),
      onOpenProfile: (_) async {
        repository.followedProfiles.add(creator.id);
      },
      onOpenHashtag: (_) async {},
    );
    await tester.pumpWidget(
      MaterialApp(
        home: BlocProvider.value(
          value: cubit..load(),
          child: DiscoveryFeedScreen(request: request),
        ),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.byTooltip('Follow ${creator.displayName}'), findsOneWidget);

    await tester.tap(find.byTooltip('Open profile'));
    await tester.pumpAndSettle();

    expect(find.byTooltip('Follow ${creator.displayName}'), findsNothing);
  });
}
