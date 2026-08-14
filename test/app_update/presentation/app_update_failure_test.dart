import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';

import '../support/update_domain_fixture.dart';
import '../../support/app_update_cubit_harness.dart';

void main() {
  late AppUpdateCubitHarness harness;

  blocTest<AppUpdateCubit, AppUpdateState>(
    'keeps a catalog failure nonfatal and allows a later retry',
    setUp: () {
      harness = AppUpdateCubitHarness();
      harness.catalog.failure = const AppFailure('Catalog unavailable.');
    },
    build: () => harness.build(),
    act: (cubit) async {
      await cubit.start();
      harness.catalog.failure = null;
      harness.catalog.release = sampleStableRelease(
        versionName: '0.0.1',
        versionCode: 1,
      );
      await cubit.checkNow();
    },
    expect: () => [
      isA<AppUpdateCheckingState>(),
      isA<AppUpdateFailureState>().having(
        (state) => state.message,
        'message',
        'Catalog unavailable.',
      ),
      isA<AppUpdateCheckingState>(),
      isA<AppUpdateCurrentState>(),
    ],
  );
}
