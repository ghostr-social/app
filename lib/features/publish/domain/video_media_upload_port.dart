import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/features/publish/domain/uploaded_video_media.dart';

abstract interface class VideoMediaUploadPort {
  Future<UploadedVideoMedia> upload(SelectedMedia media);
}
