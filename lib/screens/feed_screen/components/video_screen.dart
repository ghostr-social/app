import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:get_it/get_it.dart';
import 'package:ghostr/screens/feed_screen/components/profile_view.dart';

import '../../../data/video.dart';
import '../../../widgets/bottom_bar.dart';
import '../../feed_viewmodel.dart';
import '../../messages_screen.dart';
import '../../profile_screen.dart';
import '../../search_screen.dart';
import 'feed_videos.dart';

Widget videoScreen(UserData? userData) {
  var feed = GetIt.instance<FeedViewModel>();
  return Scaffold(
    backgroundColor: feed.actualScreen == 0 ? Colors.black : Colors.white,
    body: Stack(
      children: [
        PageView.builder(
          itemCount: 2,
          onPageChanged: (value) {
            print(value);
            if (value == 1) {
              SystemChrome.setSystemUIOverlayStyle(SystemUiOverlayStyle.dark);
            } else {
              SystemChrome.setSystemUIOverlayStyle(SystemUiOverlayStyle.light);
            }
          },
          itemBuilder: (context, index) {
            if (index == 0) {
              return scrollFeed();
            } else {
              return profileView(userData);
            }
          },
        )
      ],
    ),
  );
}

Widget scrollFeed() {
  return Column(
    mainAxisAlignment: MainAxisAlignment.end,
    children: [
      Expanded(child: currentScreen()),
      BottomBar(),
    ],
  );
}

Widget currentScreen() {
  var feed = GetIt.instance<FeedViewModel>();
  switch (feed.actualScreen) {
    case 0:
      return feedVideos(feed);
    case 1:
      return SearchScreen();
    case 2:
      return MessagesScreen();
    case 3:
      return ProfileScreen();
    default:
      return feedVideos(feed);
  }
}
