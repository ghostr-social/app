import 'package:flutter/material.dart';
import 'package:get_it/get_it.dart';
import 'package:video_player/video_player.dart';

import '../../../data/video.dart';
import '../../../widgets/actions_toolbar.dart';
import '../../../widgets/video_description.dart';
import '../../feed_viewmodel.dart';

Widget videoCard(Video video, int index) {
  final feedViewModel = GetIt.instance<FeedViewModel>();

  // The single global player
  final activeController = feedViewModel.player;

  // Check if this card's index is the "current video"
  final isActive = (index == feedViewModel.currentVideoIndex);

  // Get user name or fallback
  final name = video.user.name ?? video.user.npub!;

  return Stack(
    children: [
      if (isActive && activeController != null && activeController.value.isInitialized)
      // If this card is active, show the playing video
        GestureDetector(
          onTap: () {
            if (activeController.value.isPlaying) {
              activeController.pause();
            } else {
              activeController.play();
            }
          },
          child: SizedBox.expand(
            child: FittedBox(
              fit: BoxFit.cover,
              child: SizedBox(
                width: activeController!.value.size.width,
                height: activeController.value.size.height,
                child: VideoPlayer(activeController),
              ),
            ),
          ),
        )
      else
        Container(
          color: Colors.black,
          alignment: Alignment.center,
          child: const Text("Loading / Paused", style: TextStyle(color: Colors.white)),
        ),
      // Overlay elements: Video description and actions toolbar
      Column(
        mainAxisAlignment: MainAxisAlignment.end,
        children: <Widget>[
          Row(
            mainAxisSize: MainAxisSize.max,
            crossAxisAlignment: CrossAxisAlignment.end,
            children: <Widget>[
              VideoDescription(
                username: name,
                videoTitle: video.videoTitle,
                songInfo: video.songName,
              ),
              ActionsToolbar(
                video.likes,
                video.comments,
                video.user.profilePicture ?? "",
              ),
            ],
          ),
          const SizedBox(height: 20),
        ],
      ),
    ],
  );
}
