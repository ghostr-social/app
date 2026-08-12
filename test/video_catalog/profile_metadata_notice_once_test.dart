import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';

import '../support/pending_profile_loads.dart';
import '../support/pending_profile_metadata_repository.dart';
import '../support/profile_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('early metadata error is announced once after details settle', (
    tester,
  ) async {
    final viewer = sampleSession().profile;
    final loads = PendingProfileLoads();
    final metadata = PendingProfileMetadataRepository();
    await tester.pumpWidget(
      profileScreenHarness(
        profile: loads,
        viewer: viewer,
        profileId: viewer.id,
        metadata: metadata,
      ),
    );
    await tester.pump();
    metadata.pending.completeError(
      const AppFailure('Relay metadata unavailable.'),
    );
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 300));

    expect(find.text('Relay metadata unavailable.'), findsOneWidget);
    loads.pending.complete(ProfileDetails.empty(viewer));
    await tester.pump();
    await tester.pump(const Duration(seconds: 5));
    await tester.pumpAndSettle();

    expect(find.text('Relay metadata unavailable.'), findsNothing);
  });
}
