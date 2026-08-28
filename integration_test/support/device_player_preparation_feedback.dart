import 'dart:convert';
import 'dart:math';
import 'dart:typed_data';

import 'package:ghostr/core/media/playback_asset_authority.dart';
import 'package:ghostr/features/video_inventory/domain/player_preparation_feedback_port.dart';

final class DevicePlayerPreparationFeedback
    implements PlayerPreparationFeedbackPort {
  @override
  PlayerPreparationAttempt prepare(PlaybackAssetAuthority authority) {
    final random = Random.secure();
    final bytes = Uint8List.fromList(
      List<int>.generate(16, (_) => random.nextInt(256)),
    );
    final token = base64Url.encode(bytes).replaceAll('=', '');
    return _DevicePlayerPreparationAttempt(
      PlayerPreparationAttemptToken.parse(token),
    );
  }
}

final class _DevicePlayerPreparationAttempt
    implements PlayerPreparationAttempt {
  const _DevicePlayerPreparationAttempt(this.nativeToken);

  @override
  final PlayerPreparationAttemptToken nativeToken;

  @override
  void begin() {}

  @override
  void failed(PlayerPreparationFailureKind failure) {}

  @override
  void firstFrameRendered() {}

  @override
  void initialized() {}

  @override
  void release() {}
}
