import 'package:flutter_test/flutter_test.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  test(
    'an automatic offer downloads only through tokenized acceptance',
    () async {
      final harness = AppUpdateCubitHarness();
      final cubit = harness.build();
      addTearDown(cubit.close);
      await cubit.start();
      final offered = cubit.state;

      await cubit.downloadAvailable();

      expect(cubit.state, same(offered));
      expect(harness.downloader.calls, 0);
    },
  );
}
