import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import 'fake_video_player_platform.dart';

final class CapturingVideoViewPlatform extends FakeVideoPlayerPlatform {
  final List<VideoCreationOptions> creationOptions = [];

  @override
  Future<int?> createWithOptions(VideoCreationOptions options) {
    creationOptions.add(options);
    return create(options.dataSource);
  }
}
