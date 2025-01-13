import 'package:flutter/material.dart';
import 'package:hookstr/screens/feed_screen/components/video_card.dart';

import '../../feed_viewmodel.dart';

Widget feedVideos(FeedViewModel feedViewModel) {
  return Stack(
    children: [
      PageView.builder(
        controller: PageController(
          initialPage: 0,
          viewportFraction: 1,
        ),
        itemCount: feedViewModel.videoSource?.listVideos.length,
        onPageChanged: (index) {
          index = (index % (feedViewModel.videoSource!.listVideos.length));
          feedViewModel.changeVideo(index);
        },
        scrollDirection: Axis.vertical,
        itemBuilder: (context, index) {
          index = (index % (feedViewModel.videoSource!.listVideos.length));
          return videoCard(feedViewModel.videoSource!.listVideos[index]);
        },
      ),
      SafeArea(
        child: Container(
          padding: EdgeInsets.only(top: 20),
          child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              crossAxisAlignment: CrossAxisAlignment.center,
              children: <Widget>[
                Text('Following',
                    style: TextStyle(
                        fontSize: 17.0,
                        fontWeight: FontWeight.normal,
                        color: Colors.white70)),
                SizedBox(
                  width: 7,
                ),
                Container(
                  color: Colors.white70,
                  height: 10,
                  width: 1.0,
                ),
                SizedBox(
                  width: 7,
                ),
                Text('For You',
                    style: TextStyle(
                        fontSize: 17.0,
                        fontWeight: FontWeight.bold,
                        color: Colors.white))
              ]),
        ),
      ),
    ],
  );
}
