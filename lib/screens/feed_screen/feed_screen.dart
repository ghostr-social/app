import 'package:cached_network_image/cached_network_image.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:get_it/get_it.dart';
import 'package:hookstr/screens/profile_screen.dart';
import 'package:hookstr/screens/search_screen.dart';
import 'package:stacked/stacked.dart';
import 'package:video_player/video_player.dart';

import '../../data/video.dart';
import '../../widgets/actions_toolbar.dart';
import '../../widgets/bottom_bar.dart';
import '../../widgets/video_description.dart';
import '../feed_viewmodel.dart';
import '../messages_screen.dart';
import 'components/feed_videos.dart';
import 'components/profile_view.dart';
import 'components/video_card.dart';
import 'components/video_screen.dart';

class FeedScreen extends StatefulWidget {
  const FeedScreen({super.key});

  @override
  _FeedScreenState createState() => _FeedScreenState();
}

class _FeedScreenState extends State<FeedScreen> {
  final locator = GetIt.instance;
  final feedViewModel = GetIt.instance<FeedViewModel>();
  @override
  void initState() {
    feedViewModel.loadVideo(0);
    feedViewModel.loadVideo(1);

    super.initState();
  }

  @override
  Widget build(BuildContext context) {
    return ViewModelBuilder<FeedViewModel>.reactive(
        disposeViewModel: false,
        builder: (context, model, child) => videoScreen(model.currentUserData()),
        viewModelBuilder: () => feedViewModel);
  }

  @override
  void dispose() {
    feedViewModel.videoSource?.listVideos.forEach((video) {
      video.controller?.dispose();
    });
    super.dispose();
  }
}
