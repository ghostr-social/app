import 'package:flutter/material.dart';
import 'package:video_player/video_player.dart';

import '../../../data/video.dart';
import '../../../widgets/actions_toolbar.dart';
import '../../../widgets/video_description.dart';

Widget videoCard(Video video, String profileImageURL) {
  return Stack(
    children: [
      video.controller != null
          ? GestureDetector(
        onTap: () {
          if (video.controller!.value.isPlaying) {
            video.controller?.pause();
          } else {
            video.controller?.play();
          }
        },
        child: SizedBox.expand(
            child: FittedBox(
              fit: BoxFit.cover,
              child: SizedBox(
                width: video.controller?.value.size.width ?? 0,
                height: video.controller?.value.size.height ?? 0,
                child: VideoPlayer(video.controller!),
              ),
            )),
      )
          : Container(
        color: Colors.black,
        child: Center(
          child: Text("Loading"),
        ),
      ),
      Column(
        mainAxisAlignment: MainAxisAlignment.end,
        children: <Widget>[
          Row(
            mainAxisSize: MainAxisSize.max,
            crossAxisAlignment: CrossAxisAlignment.end,
            children: <Widget>[
              VideoDescription(video.user, video.videoTitle, video.songName),
              ActionsToolbar(video.likes, video.comments, profileImageURL),
            ],
          ),
          SizedBox(height: 20)
        ],
      ),
    ],
  );
}
