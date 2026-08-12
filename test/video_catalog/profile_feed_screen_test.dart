import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/comments/presentation/comments_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_feed_screen.dart';

import '../support/fakes.dart';
import '../support/fake_video_sharing.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('a creator feed plays their shelf under their name',
      (tester) async {
    final creator = sampleCreator();
    final clip = samplePost(id: 'clip-1', caption: 'Shelf clip', creator: creator);
    final opened = <ProfileId>[];
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [samplePost(id: 'other', caption: 'Unrelated clip')],
      feed: FakeFeedScenario(
        profiles: {
          creator.id: sampleProfileDetails(profile: creator, posts: [clip]),
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
                viewer: sampleCreator(id: 'viewer-1'),
                creatorId: creator.id,
              ),
              engagement: repository,
              optional: FeedOptionalDependencies(social: repository),
            ),
            openAt: clip.id,
          )..load(),
          child: ProfileFeedScreen(
            request: ProfileFeedRequest(
              creator: creator,
              playbackPort: FakeVideoPlaybackPort(),
              shareWorkflow: FakeVideoShareWorkflow(),
              createComments: (post) => CommentsCubit(repository, post),
              onOpenProfile: (profileId) async => opened.add(profileId),
              onOpenHashtag: (_) async {},
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.widgetWithText(AppBar, creator.displayName), findsOneWidget);
    expect(find.text('Shelf clip'), findsOneWidget);
    expect(find.text('Unrelated clip'), findsNothing);

    await tester.tap(find.byTooltip('Open profile'));
    await tester.pumpAndSettle();
    expect(opened, [creator.id]);
  });
}
