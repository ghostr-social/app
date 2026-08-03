import 'package:flutter/widgets.dart';
import 'package:ghostr/core/media/video_media_source.dart';

abstract interface class VideoPlaybackPort {
  Widget buildSurface({
    required VideoMediaSource media,
    required bool isActive,
    void Function()? onPlaybackMediaReleased,
  });
}
