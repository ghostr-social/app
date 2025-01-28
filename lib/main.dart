import 'package:flutter/material.dart';
import 'package:ghostr/screens/feed_loader/feed_loader.dart';
import 'package:ghostr/service_locator.dart';
import 'package:ghostr/src/rust/frb_generated.dart';
import 'package:ghostr/src/rust/video/video.dart';

import 'configs/base.dart';


void run() async {
  // Rust initialization
  await RustLib.init();
  await ffiStartServer(
    address: serverAddress,
    maxParallelDownloads: BigInt.from(10),
    maxStorageBytes: BigInt.from(2 * 1024 * 1024 * 1024),
  );

  // Flutter initialization
  WidgetsFlutterBinding.ensureInitialized();
  setup();

  runApp(
    MaterialApp(
      debugShowCheckedModeBanner: false,
      home: FeedLoader(),
    ),
  );
}

void main() async {
  try {
    run();
  } catch (e) {
    print('Error: $e');
  }
}

