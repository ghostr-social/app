import 'dart:async';

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/ghostr_app.dart';
import 'package:ghostr/app/production_app_update.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';

import '../support/app_update_cubit_harness.dart';
import '../support/fakes.dart';
import '../support/recording_video_playback_port.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('an update offer never deactivates the playing feed video', (
    tester,
  ) async {
    final post = samplePost();
    final playback = RecordingVideoPlaybackPort();
    final updates = AppUpdateCubitHarness();
    final catalogGate = Completer<void>();
    updates.catalog.beforeResult = catalogGate.future;
    final runtime = AppUpdateRuntime(
      dependencies: AppUpdateDependencies(
        catalog: updates.catalog,
        installedApp: updates.installedApp,
        network: updates.network,
        downloader: updates.downloader,
        installer: updates.installer,
        offerHistory: updates.offerHistory,
        settings: updates.settings,
      ),
      dispose: () async {},
    );
    final dependencies = buildFakeDependencies(
      session: sampleSession(),
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: [post]),
      appUpdateRuntime: runtime,
      device: FakeDeviceDependencies(playback: playback),
    );

    await tester.pumpWidget(GhostrApp(dependencies: dependencies));
    await tester.pumpAndSettle();
    expect(playback.activity[post.media.debugLabel]!.last, isTrue);
    expect(playback.surfaceDisposals, 0);

    catalogGate.complete();
    await tester.pumpAndSettle();
    expect(find.text('Skip this version'), findsOneWidget);
    expect(playback.activity[post.media.debugLabel]!, everyElement(isTrue));
    expect(playback.surfaceDisposals, 0);

    await tester.tap(find.text('Skip this version'));
    await tester.pumpAndSettle();
    expect(playback.activity[post.media.debugLabel]!, everyElement(isTrue));
    expect(playback.surfaceDisposals, 0);
    await tester.pumpWidget(const SizedBox.shrink());
  });
}
