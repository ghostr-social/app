import 'package:ghostr/features/profile/domain/profile_image_workflow.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_workflow.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

import 'fake_activity_repository.dart';
import 'fake_incoming_video_share_port.dart';
import 'fake_media_ports.dart';
import 'fake_secret_backup_port.dart';

class FakeDeviceDependencies {
  const FakeDeviceDependencies({
    this.activity,
    this.incomingVideoShares,
    this.mediaPicker,
    this.playback,
    this.secretBackup,
    this.profileImages,
    this.sharing,
  });

  final FakeActivityRepository? activity;
  final FakeIncomingVideoSharePort? incomingVideoShares;
  final FakeMediaPickerPort? mediaPicker;
  final VideoPlaybackPort? playback;
  final FakeSecretBackupPort? secretBackup;
  final ProfileImageWorkflow? profileImages;
  final VideoShareWorkflow? sharing;
}
