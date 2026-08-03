import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/core/media/media_picker_capabilities.dart';

abstract interface class MediaPickerPort {
  MediaPickerCapabilities get capabilities;

  Future<SelectedMedia?> recoverLostVideo();
  Future<SelectedMedia?> pickFromGallery();
  Future<SelectedMedia?> captureVideo();
}
