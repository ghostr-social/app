import 'package:ghostr/core/errors/app_failure.dart';

class VideoDownloadLimitExceeded extends AppFailure {
  const VideoDownloadLimitExceeded()
      : super('The video exceeds the available cache budget.');
}
