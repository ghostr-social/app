import 'package:ghostr/features/video_catalog/data/fallback_remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';

RemoteVideoSource buildRemoteVideoDeliverySource({
  required RemoteVideoSource primary,
  required RemoteVideoSource nativeFallback,
}) {
  return FallbackRemoteVideoSource(
    primary: primary,
    fallback: nativeFallback,
  );
}
