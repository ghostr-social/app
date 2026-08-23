import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/app/production_video_playback.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/video_playback_capabilities.dart';
import 'package:ghostr/features/comments/presentation/comments_cubit.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/features/video_catalog/data/ffi_rust_feed_port.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_screen.dart';
import 'package:ghostr/platform/media/ffi_feed_focus_port.dart';
import 'package:ghostr/platform/media/ffi_playback_preparation_updates.dart';
import 'package:ghostr/platform/media/ffi_video_delivery_updates.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ndk/ndk.dart';

import 'device_playback_probe.dart';
import 'progressive_device_origin.dart';
import 'progressive_device_resources.dart';
import 'progressive_device_telemetry.dart';
import 'warp_feed_events.dart';
import 'warp_feed_focus_probe.dart';
import 'warp_feed_preparation_probe.dart';
import 'warp_feed_relay.dart';
import 'warp_feed_test_adapters.dart';

part 'warp_feed_playback_journey_ui.dart';
part 'warp_feed_playback_journey_wait.dart';

final class WarpFeedPlaybackJourney {
  WarpFeedPlaybackJourney._({
    required this.resources,
    required this.relay,
    required this.events,
    required this.cubit,
    required this.playback,
    required this.telemetry,
    required this.preparation,
    required this.focus,
  });

  static Future<WarpFeedPlaybackJourney> start() async {
    final resources = await ProgressiveDeviceResources.start(
      responseChunkDelay: const Duration(milliseconds: 4),
    );
    final events = await signedWarpFeedEvents(resources.origin);
    final relay = await WarpFeedRelay.start(events);
    try {
      return await _startEngine(resources, relay, events);
    } on Object {
      await relay.close();
      await resources.close();
      rethrow;
    }
  }

  final ProgressiveDeviceResources resources;
  final WarpFeedRelay relay;
  final List<Nip01Event> events;
  final FeedCubit cubit;
  final VideoPlaybackPort playback;
  final ProgressiveDeviceTelemetry telemetry;
  final WarpFeedPreparationMetrics preparation;
  final WarpFeedFocusProbe focus;

  Future<void> close() async {
    await cubit.close();
    await relay.close();
    await resources.close();
  }
}

Future<WarpFeedPlaybackJourney> _startEngine(
  ProgressiveDeviceResources resources,
  WarpFeedRelay relay,
  List<Nip01Event> events,
) async {
  final settings = AppSettings.defaults()
      .withRelays([RelayUrl.parse(relay.uri.toString())])
      .withSearchRelays(const []);
  final started = await FfiVideoGateway().start(
    settings,
    resources.cachePath,
    deviceIntegrationOrigin: resources.origin.origin,
  );
  if (started is VideoGatewayFailed) throw StateError(started.message);
  return _composeJourney(resources, relay, events);
}
