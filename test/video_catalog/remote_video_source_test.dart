import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';

void main() {
  test('throws a typed failure when the remote source is disabled', () {
    final source = DisabledRemoteVideoSource('Gateway offline');

    expect(source.loadRemoteFeed, throwsA(isA<AppFailure>()));
  });
}
