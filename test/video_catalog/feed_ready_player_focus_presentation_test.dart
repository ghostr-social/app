import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/platform/media/gateway_video_playback_port.dart';
import 'package:ghostr/platform/media/native_rendered_first_frame_port.dart';
import 'package:ghostr/platform/media/rendered_first_frame_protocol.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';

import '../support/fake_progressive_playback_gateway.dart';
import '../support/feed_preparation_fixture.dart';
import '../support/recording_playback_telemetry_port.dart';

void main() {
  testWidgets('a Ready focus presents before the following rapid focus', (
    tester,
  ) async {
    final fixture = FeedPreparationFixture(postCount: 3);
    final frames = StreamController<Object?>();
    final focus = _FocusProbe();
    final telemetry = RecordingPlaybackTelemetryPort();
    addTearDown(fixture.updates.close);
    addTearDown(frames.close);
    final playback = GatewayVideoPlaybackPort(
      delegate: VideoPlayerPlaybackPort(
        telemetry: telemetry,
        renderedFirstFrames: NativeRenderedFirstFramePort(
          events: frames.stream,
        ),
      ),
      gateway: FakeProgressivePlaybackGateway(
        immediatePlaybackUrl: fixture.url('p0'),
      ),
    );
    await fixture.pump(tester, playbackPort: playback, focus: focus);
    fixture.publishWindow(1, 'p0', ['p1', 'p2']);
    await fixture.settle(tester);
    _renderFrames(fixture, frames);
    await fixture.settle(tester);
    final cubit = tester.element(find.byType(PageView)).read<FeedCubit>();

    cubit.pageChanged(1);
    await tester.runAsync(() => focus.published('p1'));
    await tester.pump(const Duration(milliseconds: 16));
    await tester.runAsync(() => Future<void>.delayed(Duration.zero));
    cubit.pageChanged(2);
    await tester.runAsync(() => focus.published('p2'));
    await fixture.settle(tester);
    _renderFrames(fixture, frames);
    await fixture.settle(tester);

    expect(
      telemetry.activations.map((item) => item.videoId.value),
      containsAllInOrder(['p1', 'p2']),
    );
    expect(
      telemetry.presentations.map((item) => item.videoId.value),
      containsAllInOrder(['p1', 'p2']),
    );
  });
}

void _renderFrames(
  FeedPreparationFixture fixture,
  StreamController<Object?> frames,
) {
  for (final source in fixture.platform.sources.values) {
    final token = source.httpHeaders[warpPlaybackAttemptHeader];
    if (token != null) frames.add({'version': 1, 'attemptToken': token});
  }
}

final class _FocusProbe implements FeedFocusPort {
  final _pending = <String, Completer<void>>{};

  Future<void> published(String id) {
    return (_pending[id] ??= Completer<void>()).future;
  }

  @override
  void focusChanged(FeedFocus focus) {
    final pending = _pending[focus.current.id.value] ??= Completer<void>();
    if (!pending.isCompleted) pending.complete();
  }
}
