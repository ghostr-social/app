import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:video_player/video_player.dart';

Future<void> mountContractPlayer(
  WidgetTester tester,
  VideoPlayerController controller,
) {
  return tester.pumpWidget(
    MaterialApp(home: SizedBox.expand(child: VideoPlayer(controller))),
  );
}

Future<void> waitForController(
  WidgetTester tester,
  VideoPlayerController controller,
  bool Function(VideoPlayerValue value) condition, {
  Duration timeout = const Duration(seconds: 15),
}) async {
  final watch = Stopwatch()..start();
  while (!condition(controller.value) && watch.elapsed < timeout) {
    await pumpContractFrame(tester);
  }
  if (!condition(controller.value)) {
    fail('Video player contract timed out after $timeout: ${controller.value}');
  }
}

Future<void> pumpContractFrame(WidgetTester tester) async {
  await tester.pump(const Duration(milliseconds: 50));
  await Future<void>.delayed(const Duration(milliseconds: 20));
}
