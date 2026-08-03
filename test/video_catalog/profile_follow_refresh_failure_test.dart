import 'package:flutter_test/flutter_test.dart';

import '../support/profile_screen_harness.dart';
import '../support/refresh_failing_profile_repository.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('a completed follow stays applied when refresh fails',
      (tester) async {
    final details = sampleProfileDetails();
    final repository = RefreshFailingProfileRepository(details);
    await tester.pumpWidget(profileScreenHarness(
      profile: repository,
      viewer: sampleSession().profile,
      profileId: details.profile.id,
    ));
    await tester.pumpAndSettle();

    await tester.tap(find.text('Follow'));
    await tester.pumpAndSettle();

    expect(find.text('Following'), findsOneWidget);
  });
}
