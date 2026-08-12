import 'package:flutter/foundation.dart';
import 'package:ghostr/core/media/incoming_video_share.dart';
import 'package:ghostr/platform/sharing/android_incoming_video_share_port.dart';
import 'package:ghostr/platform/sharing/empty_incoming_video_share_port.dart';
import 'package:ghostr/platform/sharing/method_channel_incoming_video_share_gateway.dart';

IncomingVideoSharePort buildProductionIncomingVideoSharing({
  TargetPlatform? platform,
  bool? isWeb,
}) {
  final currentPlatform = platform ?? defaultTargetPlatform;
  final runningOnWeb = isWeb ?? kIsWeb;
  if (runningOnWeb || currentPlatform != TargetPlatform.android) {
    return const EmptyIncomingVideoSharePort();
  }
  return AndroidIncomingVideoSharePort(
    MethodChannelIncomingVideoShareGateway(),
  );
}
