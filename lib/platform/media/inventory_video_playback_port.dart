import 'dart:async';
import 'dart:developer';

import 'package:flutter/material.dart';
import 'package:ghostr/core/media/video_media_cache_identity.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_lease.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_priority.dart';
import 'package:ghostr/features/video_inventory/domain/video_inventory_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';
import 'package:ghostr/shared/widgets/async_state_panel.dart';
import 'package:ghostr/shared/widgets/loading_panel.dart';

part 'inventory_video_playback_surface.dart';
part 'inventory_video_playback_lease_surface.dart';

class InventoryVideoPlaybackPort implements VideoPlaybackPort {
  const InventoryVideoPlaybackPort({
    required VideoPlaybackPort delegate,
    required VideoInventoryPort inventory,
  })  : _delegate = delegate,
        _inventory = inventory;

  final VideoPlaybackPort _delegate;
  final VideoInventoryPort _inventory;

  @override
  Widget buildSurface({
    required VideoMediaSource media,
    required bool isActive,
    void Function()? onPlaybackMediaReleased,
  }) {
    return _InventoryVideoSurface(
      port: this,
      media: media,
      isActive: isActive,
      onPlaybackMediaReleased: onPlaybackMediaReleased,
    );
  }
}
