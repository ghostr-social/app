import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/core/media/video_mime_type.dart';
import 'package:ghostr/core/media/media_picker_port.dart';
import 'package:image_picker/image_picker.dart';

class ImagePickerMediaPicker implements MediaPickerPort {
  ImagePickerMediaPicker(this._imagePicker);

  final ImagePicker _imagePicker;

  @override
  Future<SelectedMedia?> recoverLostVideo() async {
    try {
      final response = await _imagePicker.retrieveLostData();
      final file = _recoveredVideo(response);
      return file == null ? null : _toMedia(file, MediaPickSource.gallery);
    } on AppFailure {
      rethrow;
    } on Object catch (error, stackTrace) {
      throw translatedBoundaryFailure(
        source: 'ghostr.media.picker',
        message: 'The interrupted video could not be recovered.',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }

  XFile? _recoveredVideo(LostDataResponse response) {
    if (response.isEmpty || response.type == RetrieveType.image) return null;
    if (response.exception == null) return response.file;
    throw const AppFailure('The interrupted video selection failed.');
  }

  @override
  Future<SelectedMedia?> captureVideo() {
    return _pick(ImageSource.camera, MediaPickSource.camera);
  }

  @override
  Future<SelectedMedia?> pickFromGallery() {
    return _pick(ImageSource.gallery, MediaPickSource.gallery);
  }

  Future<SelectedMedia?> _pick(
    ImageSource source,
    MediaPickSource mediaSource,
  ) async {
    try {
      final file = await _imagePicker.pickVideo(source: source);
      return file == null ? null : _toMedia(file, mediaSource);
    } on Object catch (error, stackTrace) {
      final name = source == ImageSource.camera ? 'Camera' : 'Gallery';
      throw translatedBoundaryFailure(
        source: 'ghostr.media.picker',
        message: '$name access was denied or unavailable.',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }

  SelectedMedia _toMedia(XFile file, MediaPickSource source) {
    return SelectedMedia(
      path: file.path,
      source: source,
      label: file.name,
      mimeType: _mimeType(file),
    );
  }

  VideoMimeType _mimeType(XFile file) {
    return VideoMimeType.tryParse(file.mimeType) ??
        VideoMimeType.fromFileName(file.name);
  }
}
