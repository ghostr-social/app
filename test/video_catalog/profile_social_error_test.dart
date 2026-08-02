import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';

import '../support/fakes.dart';
import '../support/profile_screen_harness.dart';
import '../support/sample_data.dart';

class _FailingSocialRepository extends FakeVideoCatalogRepository {
  _FailingSocialRepository({required super.forYouFeed, required super.feed});

  @override
  Future<bool> toggleFollow(String profileId) {
    throw const AppFailure('Relay rejected the follow.');
  }
}

void main() {
  testWidgets('shows a safe error when a profile social write fails',
      (tester) async {
    final creator = sampleCreator();
    final repository = _FailingSocialRepository(
      forYouFeed: [samplePost(creator: creator)],
      feed: FakeFeedScenario(profiles: {
        creator.id: sampleProfileDetails(profile: creator),
      }),
    );
    await tester.pumpWidget(profileScreenHarness(
      profile: repository,
      viewer: sampleSession().profile,
      profileId: creator.id,
    ));
    await tester.pumpAndSettle();

    await tester.tap(find.text('Follow'));
    await tester.pump();

    expect(find.text('Relay rejected the follow.'), findsOneWidget);
  });
}
