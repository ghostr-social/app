import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/network_connection_port.dart';
import 'package:ghostr/features/app_update/domain/update_package_downloader.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';

import '../../support/app_update_cubit_harness.dart';

void main() {
  late AppUpdateCubitHarness harness;

  blocTest<AppUpdateCubit, AppUpdateState>(
    'streams verified download progress over an allowed non-Wi-Fi network',
    setUp: () {
      harness = AppUpdateCubitHarness(
        connection: NetworkConnection.other,
        preferences: const AppUpdatePreferences(
          automaticChecks: true,
          downloadPolicy: UpdateDownloadPolicy.anyNetwork,
          automaticInstall: false,
        ),
      );
      harness.downloader.events = [
        const UpdateDownloadProgress(bytes: 2, totalBytes: 4),
        const UpdateDownloadProgress(bytes: 4, totalBytes: 4),
        UpdateDownloadCompleted(harness.package),
      ];
    },
    build: () => harness.build(),
    act: (cubit) => cubit.start(),
    expect: () => [
      isA<AppUpdateCheckingState>(),
      isA<AppUpdateDownloadingState>().having(
        (state) => state.bytes,
        'bytes',
        0,
      ),
      isA<AppUpdateDownloadingState>().having(
        (state) => state.fraction,
        'fraction',
        0.5,
      ),
      isA<AppUpdateDownloadingState>().having(
        (state) => state.fraction,
        'fraction',
        1.0,
      ),
      isA<AppUpdateReadyState>().having(
        (state) => state.package.path,
        'path',
        '/tmp/ghostr.apk',
      ),
    ],
  );
}
