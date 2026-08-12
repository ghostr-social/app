import 'dart:ui' show SemanticsAction;

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/social/domain/follow_outcome.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets(
    'follows an unfollowed feed creator without opening the profile',
    (tester) async {
      final semantics = tester.ensureSemantics();
      final creator = sampleCreator();
      final repository = _RecordingFollowRepository(
        forYouFeed: [samplePost(creator: creator)],
      );
      final openedProfiles = <String>[];

      await tester.pumpWidget(
        feedScreenHarness(
          repository,
          options: FeedScreenHarnessOptions(onOpenProfile: openedProfiles.add),
        ),
      );
      await tester.pumpAndSettle();

      final avatar = find.byTooltip('Open profile');
      final follow = find.byTooltip('Follow ${creator.displayName}');
      expect(creator.id, isNot(sampleSession().profile.id));
      expect(follow, findsOneWidget);
      expect(
        find.bySemanticsLabel('Follow ${creator.displayName}'),
        findsOneWidget,
      );
      final followSemantics = tester.getSemantics(
        find.bySemanticsLabel('Follow ${creator.displayName}'),
      );
      expect(
        followSemantics.getSemanticsData().hasAction(SemanticsAction.tap),
        isTrue,
      );
      expect(
        find.descendant(of: follow, matching: find.byIcon(Icons.add)),
        findsOneWidget,
      );
      final avatarRect = tester.getRect(avatar);
      final followRect = tester.getRect(follow);
      expect(followRect.center.dy, greaterThan(avatarRect.center.dy));
      // TikTok-style intersection: the badge straddles the avatar's bottom
      // edge instead of floating detached below it.
      expect(followRect.top, lessThan(avatarRect.bottom));
      expect(followRect.center.dy, moreOrLessEquals(avatarRect.bottom));
      expect(followRect.center.dx, moreOrLessEquals(avatarRect.center.dx));

      await tester.tap(follow);
      await tester.pumpAndSettle();

      expect(repository.followRequests, [creator.id]);
      expect(follow, findsNothing);
      expect(openedProfiles, isEmpty);
      semantics.dispose();
    },
  );
}

final class _RecordingFollowRepository extends FakeVideoCatalogRepository {
  _RecordingFollowRepository({required super.forYouFeed});

  final followRequests = <ProfileId>[];

  @override
  Future<FollowOutcome> follow(ProfileId profileId) async {
    followRequests.add(profileId);
    followedProfiles.add(profileId);
    return FollowOutcome.newlyFollowed;
  }
}
