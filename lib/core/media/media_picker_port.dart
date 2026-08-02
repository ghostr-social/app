import 'package:ghostr/core/media/selected_media.dart';

abstract interface class MediaPickerPort {
  Future<SelectedMedia?> recoverLostVideo();
  Future<SelectedMedia?> pickFromGallery();
  Future<SelectedMedia?> captureVideo();
}
