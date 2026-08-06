import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_origin.dart';
import 'package:ghostr/features/video_sharing/presentation/video_share_cubit.dart';

import '../support/fake_video_sharing.dart';
import '../support/sample_data.dart';

void main() {
  test('does not emit a share completion after disposal', () async {
    final pending = Completer<void>();
    final cubit = VideoShareCubit(FakeVideoShareWorkflow(pending: pending));

    final sharing = cubit.share(samplePost(), origin: VideoShareOrigin.zero);
    await cubit.close();
    pending.complete();
    await sharing;

    expect(cubit.state, isA<VideoShareInProgress>());
  });
}
