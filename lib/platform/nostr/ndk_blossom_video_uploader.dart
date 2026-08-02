import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/features/publish/domain/video_media_upload_port.dart';
import 'package:ghostr/features/publish/domain/uploaded_video_media.dart';
import 'package:ghostr/features/settings/domain/blossom_server_url.dart';
import 'package:ghostr/platform/nostr/blossom_upload_result_mapper.dart';
import 'package:ndk/ndk.dart';

class NdkBlossomVideoUploader implements VideoMediaUploadPort {
  NdkBlossomVideoUploader({
    required Ndk ndk,
    required List<BlossomServerUrl> servers,
    BlossomUploadResultMapper mapper = const BlossomUploadResultMapper(),
  })  : _ndk = ndk,
        _servers = servers.map((server) => server.value).toList(),
        _mapper = mapper;

  final Ndk _ndk;
  final List<String> _servers;
  final BlossomUploadResultMapper _mapper;

  @override
  Future<UploadedVideoMedia> upload(SelectedMedia media) async {
    try {
      final progress = await _ndk.files
          .uploadFromFile(
            filePath: media.path,
            serverUrls: _servers,
            contentType: media.mimeType.value,
          )
          .last;
      return _mapper.map(
        progress,
        fallbackMimeType: media.mimeType.value,
      );
    } on Object catch (error, stackTrace) {
      if (error is AppFailure) rethrow;
      throw translatedBoundaryFailure(
        source: 'ghostr.nostr.blossom',
        message: 'Could not upload the video to Blossom.',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }
}
