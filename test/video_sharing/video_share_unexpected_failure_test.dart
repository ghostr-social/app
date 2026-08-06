import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_origin.dart';
import 'package:ghostr/features/video_sharing/presentation/video_share_cubit.dart';

import '../support/fake_video_sharing.dart';
import '../support/sample_data.dart';

void main() {
  test('translates an unexpected sharing failure for the viewer', () async {
    final cubit = VideoShareCubit(
      FakeVideoShareWorkflow(failure: StateError('platform broke')),
    );

    await cubit.share(samplePost(), origin: VideoShareOrigin.zero);

    final failed = cubit.state as VideoShareFailed;
    expect(failed.message, 'Could not share this video.');
    await cubit.close();
  });
}
