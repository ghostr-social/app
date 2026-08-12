import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/production_app_update.dart';
import 'package:ghostr/features/app_update/presentation/app_update_dependencies.dart';

import '../support/app_update_cubit_harness.dart';

void main() {
  test('shares one disposal across concurrent owners', () async {
    final updates = AppUpdateCubitHarness();
    final release = Completer<void>();
    var disposeCalls = 0;
    final runtime = AppUpdateRuntime(
      dependencies: AppUpdateDependencies(
        catalog: updates.catalog,
        installedApp: updates.installedApp,
        network: updates.network,
        downloader: updates.downloader,
        installer: updates.installer,
        settings: updates.settings,
      ),
      dispose: () async {
        disposeCalls += 1;
        await release.future;
      },
    );

    final firstOwner = runtime.dispose();
    final secondOwner = runtime.dispose();
    expect(disposeCalls, 1);

    release.complete();
    await Future.wait([firstOwner, secondOwner]);
    expect(disposeCalls, 1);
  });
}
