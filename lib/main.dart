import 'package:flutter/material.dart';
import 'package:ghostr/screens/feed_screen/feed_screen.dart';
import 'package:ghostr/service_locator.dart';
import 'package:ghostr/src/rust/frb_generated.dart';
import 'package:ghostr/src/rust/video/video.dart';

import 'configs/base.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();
  ffiStartServer(address: serverAddress);
  setup();

  runApp(
    MaterialApp(
      debugShowCheckedModeBanner: false,
      home: FeedScreen(),
    ),
  );
}

