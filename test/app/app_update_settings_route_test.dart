import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/app/app_update_scope.dart';
import 'package:ghostr/app/router/app_router.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';

import '../support/app_update_cubit_harness.dart';
import '../support/fakes.dart';

void main() {
  testWidgets('settings Check now invokes the global updater', (tester) async {
    final update = AppUpdateCubitHarness(
      preferences: const AppUpdatePreferences(
        automaticChecks: false,
        downloadPolicy: UpdateDownloadPolicy.wifiOnly,
        automaticInstall: true,
      ),
    );
    final cubit = update.build();
    final controllers = AppControllerFactory(
      buildFakeDependencies(
        catalogRepository: FakeVideoCatalogRepository(forYouFeed: const []),
      ),
    );

    await tester.pumpWidget(
      AppUpdateScope(
        create: () => cubit,
        child: MaterialApp(
          home: Builder(
            builder: (context) => ElevatedButton(
              onPressed: () =>
                  Navigator.of(context).push(AppRouter.settings(controllers)),
              child: const Text('Settings'),
            ),
          ),
        ),
      ),
    );
    await tester.tap(find.text('Settings'));
    await tester.pumpAndSettle();
    final checkNow = find.byKey(const Key('check-for-updates-button'));
    await tester.scrollUntilVisible(
      checkNow,
      400,
      scrollable: find.byType(Scrollable).first,
    );
    await Scrollable.ensureVisible(tester.element(checkNow), alignment: 0.5);
    await tester.pumpAndSettle();
    await tester.tap(checkNow);
    await tester.pumpAndSettle();

    expect(update.catalog.calls, 1);
    expect(find.text('Ghostr 0.0.2 is available.'), findsOneWidget);
    await tester.pumpWidget(const SizedBox.shrink());
  });
}
