import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_screen.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ndk/ndk.dart';
import 'package:ghostr/src/rust/api/warp_evidence_control.dart';
import 'package:integration_test/integration_test.dart';

import 'device_playback_probe.dart';
import 'device_qoe_targets.dart';
import 'live_video_log.dart';
import 'live_video_corpus.dart';
import 'live_direct_playback.dart';
import 'live_origin_probe.dart';
import 'live_video_runtime.dart';
import 'live_motion_window.dart';
import 'live_relay_read.dart';
import 'live_comparison_surface.dart';

part 'live_video_journey_wait.dart';
part 'live_video_journey_browse.dart';
part 'live_video_journey_sample.dart';
part 'live_video_journey_evidence.dart';
part 'live_video_journey_controls.dart';
part 'live_video_journey_motion.dart';
part 'live_video_journey_pins.dart';
part 'live_video_journey_pairs.dart';

final class LiveVideoJourney {
  LiveVideoJourney(this.binding, this.tester) {
    runtime = LiveVideoRuntime(log);
  }

  final IntegrationTestWidgetsFlutterBinding binding;
  final WidgetTester tester;
  final log = LiveVideoLog();
  late final LiveVideoRuntime runtime;
  final failures = <String>[];
  final samples = <Map<String, Object?>>[];
  final evidence = <Object?>[];
  final visited = <String>{};
  final corpus = LiveVideoCorpus.fromJson(
    const String.fromEnvironment('LIVE_VIDEO_PRIOR_CORPUS', defaultValue: '{}'),
  );
  FeedCubit? get cubit {
    final elements = find.byType(FeedScreen).evaluate();
    return elements.isEmpty ? null : elements.first.read<FeedCubit>();
  }

  Future<void> run() async {
    final semantics = tester.ensureSemantics();
    addTearDown(semantics.dispose);
    await tester.pumpWidget(await runtime.start());
    final ready = await waitUntil(() => cubit?.state is FeedLoaded);
    log.add('initial_feed', {'ready': ready, 'state': '${cubit?.state}'});
    if (log.watch.elapsed > const Duration(seconds: 5)) {
      failures.add('App-to-feed exceeded 5 seconds: ${log.watch.elapsed}.');
    }
    if (!ready) {
      failures.add('No real feed within 30 seconds after bootstrap.');
      return;
    }
    expect(find.text('For You'), findsOneWidget);
    if (const String.fromEnvironment('LIVE_VIDEO_EVENT_IDS').isNotEmpty) {
      await replayPins();
    } else {
      await browse();
      await warmReturn();
      await rapidSwipes();
      await controls();
    }
  }

  Future<void> finish() async {
    await captureEvidence();
    log.add('result', {'samples': samples.length, 'failures': failures});
    binding.reportData = {
      ...log.report(),
      'samples': samples,
      'failures': failures,
      'warpEvidence': evidence,
    };
    await tester.pumpWidget(const SizedBox.shrink());
    await runtime.close();
  }
}
