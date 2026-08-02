import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';

import '../support/fakes.dart';
import '../support/profile_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('uses an app-safe message for an unexpected profile load error',
      (tester) async {
    await tester.pumpWidget(profileScreenHarness(
      profile: _UnexpectedProfileRepository(),
      viewer: sampleSession().profile,
      profileId: sampleCreator().id,
    ));
    await tester.pumpAndSettle();

    expect(find.text('Could not load this profile.'), findsOneWidget);
  });
}

class _UnexpectedProfileRepository extends FakeVideoCatalogRepository {
  _UnexpectedProfileRepository() : super(forYouFeed: []);

  @override
  Future<ProfileDetails> loadProfile(
    ProfileSummary viewer,
    ProfileId profileId,
  ) {
    throw StateError('profile unavailable');
  }
}
