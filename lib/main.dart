import 'package:flutter/material.dart';
import 'package:hookstr/screens/feed_screen.dart';
import 'package:hookstr/service_locator.dart';
import 'package:nostr_sdk/nostr_sdk.dart';

void main() async {
  await NostrSdk.init();
  setup();

  // Wrap your FeedScreen with MaterialApp
  runApp(
    MaterialApp(
      debugShowCheckedModeBanner: false,
      home: FeedScreen(),
    ),
  );
}