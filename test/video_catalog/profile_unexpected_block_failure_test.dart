import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/fakes.dart';
import '../support/profile_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('uses an app-safe notice for an unexpected profile block error',
      (tester) async {
    final creator = sampleCreator();
    final repository = _UnexpectedBlockRepository(
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

    expect(find.text('Could not update this block.'), findsOneWidget);
  });
}

class _UnexpectedBlockRepository extends FakeVideoCatalogRepository {
  _UnexpectedBlockRepository(ProfileId id, ProfileDetails details)
      : super(
          forYouFeed: [],
          feed: FakeFeedScenario(profiles: {id: details}),
        );

  @override
  Future<bool> toggleBlock(ProfileId profileId) {
    throw StateError('signer unavailable');
  }
}
