import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../support/pending_profile_loads.dart';
import '../support/profile_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('cached profile survives refresh failure with retry', (
    tester,
  ) async {
    final viewer = sampleSession().profile;
    final repository = FailingProfileLoads();
    await tester.pumpWidget(
      profileScreenHarness(
        profile: repository,
        viewer: viewer,
        profileId: viewer.id,
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text(viewer.displayName), findsOneWidget);
    expect(find.text('Profile refresh failed.'), findsOneWidget);
    expect(find.widgetWithText(TextButton, 'Retry'), findsOneWidget);

    await tester.tap(find.widgetWithText(TextButton, 'Retry'));
    await tester.pumpAndSettle();

    expect(repository.loadCount, 2);
    expect(find.text(viewer.displayName), findsOneWidget);
  });
}
