import 'dart:ffi';

import 'package:flutter/material.dart';
import 'package:get_it/get_it.dart';
import 'package:ghostr/data/video.dart';
import 'package:stacked/stacked.dart';

import '../feed_viewmodel.dart';
import 'components/video_screen.dart';

class FeedScreen extends StatefulWidget {
  final List<Video> initialVideos;

  const FeedScreen({super.key, required this.initialVideos});

  @override
  _FeedScreenState createState() => _FeedScreenState();
}

class _FeedScreenState extends State<FeedScreen> {
  final locator = GetIt.instance;
  final feedViewModel = GetIt.instance<FeedViewModel>();
  @override
  void initState() {
    feedViewModel.videoBank.addAll(widget.initialVideos);
    super.initState();
  }

  @override
  Widget build(BuildContext context) {
    return ViewModelBuilder<FeedViewModel>.reactive(
        disposeViewModel: false,
        builder: (context, model, child) =>
            videoScreen(model.currentUserData()),
        viewModelBuilder: () => feedViewModel);
  }

  @override
  void dispose() {
    for (var video in feedViewModel.videoBank) {
      video.controller?.dispose();
    }
    super.dispose();
  }
}
