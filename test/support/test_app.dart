import 'package:flutter/material.dart';
import 'package:ghostr/app/app_dependencies.dart';
import 'package:ghostr/app/ghostr_app.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';

Widget buildTestApp(AppDependencies dependencies, {FeedFocusSink? feedFocus}) {
  return GhostrApp(dependencies: dependencies, feedFocus: feedFocus);
}
