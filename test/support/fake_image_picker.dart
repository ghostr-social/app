import 'package:image_picker/image_picker.dart';

class FakeImagePicker extends ImagePicker {
  FakeImagePicker({this.result, this.error, this.lostData});

  final XFile? result;
  final Object? error;
  final LostDataResponse? lostData;
  ImageSource? requestedSource;

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
