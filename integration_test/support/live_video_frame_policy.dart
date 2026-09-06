import 'package:flutter_test/flutter_test.dart';

void configureLiveVideoFrames(LiveTestWidgetsFlutterBinding binding) {
  binding.framePolicy = LiveTestWidgetsFlutterBindingFramePolicy.benchmarkLive;
}
