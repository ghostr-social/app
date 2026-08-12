import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/production_app_update.dart';
import 'package:ghostr/features/app_update/presentation/app_update_dependencies.dart';

import '../support/app_update_cubit_harness.dart';

void main() {
  test('shares one failed disposal across concurrent owners', () async {
    final updates = AppUpdateCubitHarness();
    final failure = StateError('dispose failed');
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
      dispose: () {
        disposeCalls += 1;
        throw failure;
      },
    );

    final firstOwner = expectLater(runtime.dispose(), throwsA(same(failure)));
    final secondOwner = expectLater(runtime.dispose(), throwsA(same(failure)));

    await Future.wait([firstOwner, secondOwner]);
    expect(disposeCalls, 1);
  });
}
