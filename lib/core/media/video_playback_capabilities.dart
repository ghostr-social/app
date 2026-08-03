import 'package:ghostr/core/media/video_media_source.dart';

final class VideoPlaybackCapabilities {
  const VideoPlaybackCapabilities._({
    required this.supportsLocalFiles,
    required Set<VideoMediaDelivery> remoteDeliveries,
  }) : _remoteDeliveries = remoteDeliveries;

  static const none = VideoPlaybackCapabilities._(
    supportsLocalFiles: false,
    remoteDeliveries: {},
  );
  static const progressiveOnly = VideoPlaybackCapabilities._(
    supportsLocalFiles: true,
    remoteDeliveries: {VideoMediaDelivery.progressive},
  );
  static const progressiveAndHls = VideoPlaybackCapabilities._(
    supportsLocalFiles: true,
    remoteDeliveries: {
      VideoMediaDelivery.progressive,
      VideoMediaDelivery.hls,
    },
  );

  final bool supportsLocalFiles;
  final Set<VideoMediaDelivery> _remoteDeliveries;

  bool get supportsAny => supportsLocalFiles || _remoteDeliveries.isNotEmpty;
  bool get supportsHls => _remoteDeliveries.contains(VideoMediaDelivery.hls);

  bool supports(VideoMediaSource media) {
    final delivery = media.remoteDelivery;
    return delivery == null
        ? media.isLocal && supportsLocalFiles
        : _remoteDeliveries.contains(delivery);
  }

  VideoPlaybackCapabilities without(VideoMediaDelivery delivery) {
    if (!_remoteDeliveries.contains(delivery)) return this;
    return VideoPlaybackCapabilities._(
      supportsLocalFiles: supportsLocalFiles,
      remoteDeliveries: _remoteDeliveries.difference({delivery}),
    );
  }
}
