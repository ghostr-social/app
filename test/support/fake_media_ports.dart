import 'package:flutter/material.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/media_picker_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

class FakeMediaPickerPort implements MediaPickerPort {
  FakeMediaPickerPort({
    this.galleryMedia,
    this.cameraMedia,
    this.galleryFailure,
    this.cameraFailure,
    this.recoveredMedia,
  });

  SelectedMedia? galleryMedia;
  SelectedMedia? cameraMedia;
  AppFailure? galleryFailure;
  AppFailure? cameraFailure;
  SelectedMedia? recoveredMedia;
  int galleryPickCount = 0;

  @override
  Future<SelectedMedia?> captureVideo() async {
    if (cameraFailure case final AppFailure failure) throw failure;
    return cameraMedia;
  }

  @override
  Future<SelectedMedia?> pickFromGallery() async {
    galleryPickCount += 1;
    if (galleryFailure case final AppFailure failure) throw failure;
    return galleryMedia;
  }

  @override
  Future<SelectedMedia?> recoverLostVideo() async => recoveredMedia;
}

class FakeVideoPlaybackPort implements VideoPlaybackPort {
  @override
  Widget buildSurface({
    required VideoMediaSource media,
    required bool isActive,
  }) {
    return ColoredBox(
      color: isActive ? Colors.black : Colors.black54,
      child: Center(
        child: Text(
          media.debugLabel,
          textAlign: TextAlign.center,
        ),
      ),
    );
  }
}
