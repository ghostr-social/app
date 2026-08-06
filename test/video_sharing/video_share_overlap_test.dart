import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_origin.dart';
import 'package:ghostr/features/video_sharing/presentation/video_share_cubit.dart';

import '../support/fake_video_sharing.dart';
import '../support/sample_data.dart';

void main() {
  test('ignores another share intent while a download is pending', () async {
    final pending = Completer<void>();
    final workflow = FakeVideoShareWorkflow(pending: pending);
    final cubit = VideoShareCubit(workflow);
    final post = samplePost();

    final first = cubit.share(post, origin: VideoShareOrigin.zero);
    await cubit.share(post, origin: VideoShareOrigin.zero);

    expect(workflow.requests, hasLength(1));
    pending.complete();
    await first;
    await cubit.close();
  });
}
