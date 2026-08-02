import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/fakes.dart';
import '../support/profile_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('preserves an app-safe profile block failure', (tester) async {
    final creator = sampleCreator();
    final repository = _RejectedBlockRepository(
      creator.id,
      sampleProfileDetails(profile: creator),
    );
    await tester.pumpWidget(profileScreenHarness(
      profile: repository,
      viewer: sampleSession().profile,
      profileId: creator.id,
    ));
    await tester.pumpAndSettle();

    await tester.tap(find.text('Block'));
    await tester.pump();

    expect(find.text('Relay rejected the block.'), findsOneWidget);
  });
}

class _RejectedBlockRepository extends FakeVideoCatalogRepository {
  _RejectedBlockRepository(ProfileId id, ProfileDetails details)
      : super(
          forYouFeed: [],
          feed: FakeFeedScenario(profiles: {id: details}),
        );

  @override
  Future<bool> toggleBlock(ProfileId profileId) {
    throw const AppFailure('Relay rejected the block.');
  }
}
