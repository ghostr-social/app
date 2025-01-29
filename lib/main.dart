import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:ghostr/screens/feed_loader/feed_loader.dart';
import 'package:ghostr/service_locator.dart';
import 'package:ghostr/src/rust/frb_generated.dart';
import 'package:ghostr/src/rust/video/video.dart';

import 'configs/base.dart';


void run() async {
  // Rust initialization
  await RustLib.init();
  var endpoint = await ffiStartServer(
    maxParallelDownloads: BigInt.from(10),
    maxStorageBytes: BigInt.from(2 * 1024 * 1024 * 1024),
  );
  if (kDebugMode) print('Server started at $endpoint');


  // Flutter initialization
  WidgetsFlutterBinding.ensureInitialized();
  setup(endpoint);

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
    if (kDebugMode) print('Error: $e');
  }
}

