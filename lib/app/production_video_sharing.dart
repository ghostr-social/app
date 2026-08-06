import 'package:ghostr/features/video_inventory/domain/progressive_playback_gateway_port.dart';
import 'package:ghostr/features/video_sharing/data/default_video_share_workflow.dart';
import 'package:ghostr/features/video_sharing/domain/video_file_share_port.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_workflow.dart';
import 'package:ghostr/platform/sharing/gateway_video_file_downloader.dart';
import 'package:ghostr/platform/sharing/http_video_file_transfer.dart';
import 'package:ghostr/platform/sharing/share_plus_video_file_port.dart';
import 'package:ghostr/platform/media/ffi_progressive_playback_gateway.dart';
import 'package:http/http.dart' as http;
import 'package:path_provider/path_provider.dart';

VideoShareWorkflow buildProductionVideoSharing({
  ProgressivePlaybackGatewayPort gateway =
      const FfiProgressivePlaybackGateway(),
  VideoFileTransfer? transfer,
  VideoFileSharePort? sharePort,
  TemporaryDirectoryPath temporaryDirectoryPath = temporaryVideoDirectory,
}) {
  return DefaultVideoShareWorkflow(
    downloader: GatewayVideoFileDownloader(
      gateway: gateway,
      transfer: transfer ?? HttpVideoFileTransfer(http.Client()),
      temporaryDirectoryPath: temporaryDirectoryPath,
    ),
    sharePort: sharePort ?? SharePlusVideoFilePort(),
  );
}

// coverage:ignore-start
Future<String> temporaryVideoDirectory() async {
  return (await getTemporaryDirectory()).path;
}

// coverage:ignore-end
