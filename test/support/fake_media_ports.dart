import 'package:flutter/material.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/media/media_picker_capabilities.dart';
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
    this.recoveredMediaFuture,
  });

  SelectedMedia? galleryMedia;
  SelectedMedia? cameraMedia;
  AppFailure? galleryFailure;
  AppFailure? cameraFailure;
  SelectedMedia? recoveredMedia;
  Future<SelectedMedia?>? recoveredMediaFuture;
  @override
  MediaPickerCapabilities capabilities =
      const MediaPickerCapabilities.allSupported();
  int galleryPickCount = 0;
  int cameraPickCount = 0;

  @override
  Future<SelectedMedia?> captureVideo() async {
    cameraPickCount += 1;
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
  Future<SelectedMedia?> recoverLostVideo() =>
      recoveredMediaFuture ?? Future.value(recoveredMedia);
}

class FakeVideoPlaybackPort implements VideoPlaybackPort {
  @override
  Widget buildSurface({
    required VideoMediaSource media,
    required bool isActive,
    void Function()? onPlaybackMediaReleased,
  }) {
    return _ReleaseOnDispose(
      key: ValueKey(media.debugLabel),
      onReleased: onPlaybackMediaReleased,
      child: ColoredBox(
        color: isActive ? Colors.black : Colors.black54,
        child: Center(
          child: Text(media.debugLabel, textAlign: TextAlign.center),
        ),
      ),
    );
  }
}

class _ReleaseOnDispose extends StatefulWidget {
  const _ReleaseOnDispose({
    required this.onReleased,
    required this.child,
    super.key,
  });

  final void Function()? onReleased;
  final Widget child;

  @override
  State<_ReleaseOnDispose> createState() => _ReleaseOnDisposeState();
}

class _ReleaseOnDisposeState extends State<_ReleaseOnDispose> {
  @override
  Widget build(BuildContext context) => widget.child;

  @override
  void dispose() {
    widget.onReleased?.call();
    super.dispose();
  }
}
