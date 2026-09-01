import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/network/delivery_network_status.dart';
import 'package:ghostr/features/settings/domain/data_usage_level.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import 'device_playback_probe.dart';
import 'progressive_device_origin.dart';
import 'progressive_device_resources.dart';
import 'warp_feed_event_config.dart';
import 'warp_feed_events.dart';
import 'warp_feed_production_graph.dart';
import 'warp_feed_production_graph_build.dart';
import 'warp_feed_player_stage_probe.dart';
import 'warp_feed_relay.dart';
import 'warp_feed_surface.dart';
import 'warp_offline_restart_account.dart';
import 'warp_offline_restart_manifest.dart';
import 'warp_offline_restart_snapshot.dart';
import 'warp_offline_restart_storage.dart';

part 'warp_offline_restart_fixture_acceptance.dart';
part 'warp_offline_restart_fixture_evidence.dart';
part 'warp_offline_restart_fixture_promotion.dart';
part 'warp_offline_restart_fixture_restore.dart';
part 'warp_offline_restart_fixture_seed.dart';
part 'warp_offline_restart_fixture_wait.dart';

final class WarpOfflineRestartFixture {
  WarpOfflineRestartFixture._({
    required this.resources,
    required this.graph,
    required this.storage,
    required this.manifest,
    this.relay,
  });

  static Future<WarpOfflineRestartFixture> seed() => _startOfflineSeed();

  static Future<WarpOfflineRestartFixture> restore() => _startOfflineRestore();

  final ProgressiveDeviceResources resources;
  final WarpFeedProductionGraph graph;
  final WarpOfflineRestartStorage storage;
  final WarpOfflineRestartManifest manifest;
  final WarpFeedRelay? relay;
  var _closed = false;

  Widget get app => MaterialApp(home: WarpFeedSurface(graph: graph));
  FeedCubit get cubit => graph.cubit;
  List<String> get originBodyRequestedIds => resources.origin.bodyRequestedIds;

  bool get hasCachedSignedPost {
    final state = cubit.state;
    if (state is! FeedLoaded) return false;
    return state.posts.any((post) {
      return post.id.value == manifest.eventId &&
          post.nostrReference?.signedEvent != null;
    });
  }

  void load() => unawaited(cubit.load());

  Future<void> close() async {
    if (_closed) return;
    _closed = true;
    try {
      await graph.close();
    } finally {
      try {
        await relay?.close();
      } finally {
        await resources.close();
      }
    }
  }

  Future<void> closeAndDelete() async {
    try {
      await close();
    } finally {
      await storage.delete();
    }
  }
}
