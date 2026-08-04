import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';

/// The delivery feed is the relay path alone: the viewer-blind native
/// fallback is retired (plan §4 step 10), and its accepted regression
/// is that a failing or empty relay outcome is served as-is.
RemoteVideoSource buildRemoteVideoDeliverySource({
  required RemoteVideoSource primary,
}) {
  return primary;
}
