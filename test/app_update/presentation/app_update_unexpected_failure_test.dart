import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  late AppUpdateCubitHarness harness;

  blocTest<AppUpdateCubit, AppUpdateState>(
    'translates an unexpected boundary exception into a safe failure',
    setUp: () {
      harness = AppUpdateCubitHarness();
      harness.catalog.failure = StateError('transport internals');
    },
    build: () => harness.build(),
    act: (cubit) => cubit.start(),
    expect: () => [
      isA<AppUpdateCheckingState>(),
      isA<AppUpdateFailureState>().having(
        (state) => state.message,
        'message',
        'Could not complete the update operation.',
      ),
    ],
  );
}
