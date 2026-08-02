import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/features/publish/domain/video_media_upload_port.dart';
import 'package:ghostr/features/publish/domain/uploaded_video_media.dart';

class FakeVideoMediaUploadPort implements VideoMediaUploadPort {
  FakeVideoMediaUploadPort(this.result);

  final UploadedVideoMedia result;
  SelectedMedia? uploadedMedia;

  @override
  Future<UploadedVideoMedia> upload(SelectedMedia media) async {
    uploadedMedia = media;
    return result;
  }
}
