import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/network_connection_port.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  blocTest<AppUpdateCubit, AppUpdateState>(
    'reports offline automatic download failure without blocking the app',
    build: () => AppUpdateCubitHarness(
      connection: NetworkConnection.offline,
      preferences: const AppUpdatePreferences(
        automaticChecks: true,
        downloadPolicy: UpdateDownloadPolicy.anyNetwork,
        automaticInstall: false,
      ),
    ).build(),
    act: (cubit) => cubit.start(),
    expect: () => [
      isA<AppUpdateCheckingState>(),
      isA<AppUpdateFailureState>().having(
        (state) => state.message,
        'message',
        'Connect to the internet to download the update.',
      ),
    ],
  );
}
