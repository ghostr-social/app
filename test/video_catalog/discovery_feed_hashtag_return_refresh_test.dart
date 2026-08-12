import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/comments/presentation/comments_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/query_video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/presentation/discovery_feed_screen.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/caption_text.dart';

import '../support/fakes.dart';
import '../support/fake_video_sharing.dart';
import '../support/follow_profile_workflow.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('returning from a nested hashtag refreshes follows', (
    tester,
  ) async {
    final creator = sampleCreator();
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [samplePost(caption: 'Try #dance', creator: creator)],
    );
    final cubit = FeedCubit(
      FeedDependencies(
        viewerId: sampleSession().profile.id,
        feed: QueryVideoFeedRepository(search: repository, query: '#music'),
        engagement: repository,
        followProfile: testFollowProfileWorkflow(repository),
        optional: FeedOptionalDependencies(social: repository),
      ),
    );
    addTearDown(cubit.close);
    await tester.pumpWidget(
      MaterialApp(
        home: BlocProvider.value(
          value: cubit..load(),
          child: DiscoveryFeedScreen(
            request: DiscoveryFeedRequest(
              query: '#music',
              playbackPort: FakeVideoPlaybackPort(),
              shareWorkflow: FakeVideoShareWorkflow(),
              createComments: (post) => CommentsCubit(repository, post),
              onOpenProfile: (_) async {},
              onOpenHashtag: (_) async {
                repository.followedProfiles.add(creator.id);
              },
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();

    _hashtagRecognizer(tester).onTap!();
    await tester.pumpAndSettle();

    expect(find.byTooltip('Follow ${creator.displayName}'), findsNothing);
  });
}

TapGestureRecognizer _hashtagRecognizer(WidgetTester tester) {
  final caption = tester.widget<Text>(
    find.descendant(of: find.byType(CaptionText), matching: find.byType(Text)),
  );
  TapGestureRecognizer? recognizer;
  caption.textSpan!.visitChildren((span) {
    if (span is TextSpan && span.text == '#dance') {
      recognizer = span.recognizer as TapGestureRecognizer?;
      return false;
    }
    return true;
  });
  return recognizer!;
}
