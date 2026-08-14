import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/network_connection_port.dart';
import 'package:ghostr/features/app_update/presentation/app_update_offer_overlay.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';

import '../support/app_update_cubit_harness.dart';

void main() {
  testWidgets('a failed Update keeps a visible retryable offer', (
    tester,
  ) async {
    final harness = AppUpdateCubitHarness(
      connection: NetworkConnection.offline,
      preferences: const AppUpdatePreferences(
        automaticChecks: true,
        downloadPolicy: UpdateDownloadPolicy.anyNetwork,
        automaticInstall: false,
      ),
    );
    final cubit = harness.build();
    addTearDown(cubit.close);
    await cubit.start();
    await tester.pumpWidget(
      MaterialApp(
        home: AppUpdateOfferOverlay(
          cubit: cubit,
          child: const Scaffold(body: Text('Video')),
        ),
      ),
    );

    await tester.tap(find.widgetWithText(FilledButton, 'Update'));
    await tester.pumpAndSettle();

    expect(
      find.text('Connect to the internet to download the update.'),
      findsOneWidget,
    );
    expect(
      find.bySemanticsLabel('Connect to the internet to download the update.'),
      findsOneWidget,
    );
    expect(find.widgetWithText(FilledButton, 'Update'), findsOneWidget);
  });
}
