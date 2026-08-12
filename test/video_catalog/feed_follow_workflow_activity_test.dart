import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/social/domain/follow_outcome.dart';
import 'package:ghostr/features/video_catalog/domain/follow_profile_workflow.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('records activity when a feed creator is newly followed', () async {
    final social = _NewFollowSocial();
    final activity = FakeActivityRepository();
    final workflow = DefaultFollowProfileWorkflow(
      social: social,
      activity: activity,
      clock: () => DateTime.utc(2026, 8, 12),
      failureReporter: RecordingFailureReporter(),
    );
    final creator = sampleCreator();

    final outcome = await workflow.follow(creator);

    expect(outcome, FollowOutcome.newlyFollowed);
    expect(social.requests, [creator.id]);
    expect(
      (await activity.load()).single.description.title,
      'Started following ${creator.displayName}',
    );
  });
}

final class _NewFollowSocial extends FakeVideoCatalogRepository {
  _NewFollowSocial() : super(forYouFeed: []);

  final requests = <ProfileId>[];

  @override
  Future<FollowOutcome> follow(ProfileId profileId) async {
    requests.add(profileId);
    return FollowOutcome.newlyFollowed;
  }
}
