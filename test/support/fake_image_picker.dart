import 'package:image_picker/image_picker.dart';

class FakeImagePicker extends ImagePicker {
  FakeImagePicker({this.result, this.error, this.lostData});

  final XFile? result;
  final Object? error;
  final LostDataResponse? lostData;
  ImageSource? requestedSource;
  ImageSource? requestedImageSource;
  double? requestedImageWidth;
  double? requestedImageHeight;
  int? requestedImageQuality;

  @override
  Future<XFile?> pickImage({
    required ImageSource source,
    double? maxWidth,
    double? maxHeight,
    int? imageQuality,
    CameraDevice preferredCameraDevice = CameraDevice.rear,
    bool requestFullMetadata = true,
  }) async {
    requestedImageSource = source;
    requestedImageWidth = maxWidth;
    requestedImageHeight = maxHeight;
    requestedImageQuality = imageQuality;
    if (error != null) throw error!;
    return result;
  }

  @override
  Future<XFile?> pickVideo({
    required ImageSource source,
    CameraDevice preferredCameraDevice = CameraDevice.rear,
    Duration? maxDuration,
  }) async {
    requestedSource = source;
    if (error != null) throw error!;
    return result;
  }

  @override
  Future<LostDataResponse> retrieveLostData() async {
    return lostData ?? LostDataResponse.empty();
  }
}
