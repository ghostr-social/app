import 'package:flutter_test/flutter_test.dart';

import '../support/pending_profile_loads.dart';
import '../support/profile_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('shows cached current profile while relay refresh is pending', (
    tester,
  ) async {
    final viewer = sampleCreator(displayName: 'Cached Nora');
    await tester.pumpWidget(
      profileScreenHarness(
        profile: PendingProfileLoads(),
        viewer: viewer,
        profileId: viewer.id,
      ),
    );
    await tester.pump();

    expect(find.text('Cached Nora'), findsOneWidget);
    expect(find.text(viewer.handle), findsOneWidget);
    expect(find.text('Loading creator profile'), findsNothing);
  });
}
