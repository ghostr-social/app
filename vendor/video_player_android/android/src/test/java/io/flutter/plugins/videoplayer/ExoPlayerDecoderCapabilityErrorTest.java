// Copyright 2013 The Flutter Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package io.flutter.plugins.videoplayer;

import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.ArgumentMatchers.contains;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.isNull;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;

import androidx.media3.common.C;
import androidx.media3.common.Format;
import androidx.media3.common.PlaybackException;
import androidx.media3.exoplayer.ExoPlaybackException;
import androidx.media3.exoplayer.ExoPlayer;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.robolectric.RobolectricTestRunner;

@RunWith(RobolectricTestRunner.class)
public final class ExoPlayerDecoderCapabilityErrorTest {
  private static final Format HEVC =
      new Format.Builder()
          .setSampleMimeType("video/hevc")
          .setCodecs("hvc1.2.4.L150.90")
          .setWidth(2560)
          .setHeight(1440)
          .build();
  private static final Format AAC =
      new Format.Builder().setSampleMimeType("audio/mp4a-latm").setCodecs("mp4a.40.2").build();

  @Test
  public void decodingFailureWithExceededFormatIsDefinitive() {
    VideoPlayerCallbacks callbacks = mock(VideoPlayerCallbacks.class);
    listener(callbacks)
        .onPlayerError(
            rendererError(
                HEVC,
                C.FORMAT_EXCEEDS_CAPABILITIES,
                PlaybackException.ERROR_CODE_DECODING_FAILED));

    verify(callbacks)
        .onError(eq("VideoDecoderUnsupported"), contains("hvc1.2.4.L150.90"), any());
  }

  @Test
  public void decoderInitializationWithHandledFormatRemainsTransient() {
    VideoPlayerCallbacks callbacks = mock(VideoPlayerCallbacks.class);
    listener(callbacks)
        .onPlayerError(
            rendererError(C.FORMAT_HANDLED, PlaybackException.ERROR_CODE_DECODER_INIT_FAILED));

    verify(callbacks).onError(eq("VideoError"), anyString(), isNull());
  }

  @Test
  public void unsupportedDrmRemainsTransient() {
    VideoPlayerCallbacks callbacks = mock(VideoPlayerCallbacks.class);
    listener(callbacks)
        .onPlayerError(
            rendererError(
                HEVC, C.FORMAT_UNSUPPORTED_DRM, PlaybackException.ERROR_CODE_DECODING_FAILED));

    verify(callbacks).onError(eq("VideoError"), anyString(), isNull());
  }

  @Test
  public void audioDecoderRejectionIsNotVideoCapabilityEvidence() {
    VideoPlayerCallbacks callbacks = mock(VideoPlayerCallbacks.class);
    listener(callbacks)
        .onPlayerError(
            rendererError(
                AAC,
                C.FORMAT_EXCEEDS_CAPABILITIES,
                PlaybackException.ERROR_CODE_DECODING_FORMAT_EXCEEDS_CAPABILITIES));

    verify(callbacks).onError(eq("VideoError"), anyString(), isNull());
  }

  private static PlaybackException rendererError(int support, int code) {
    return rendererError(HEVC, support, code);
  }

  private static PlaybackException rendererError(Format format, int support, int code) {
    return ExoPlaybackException.createForRenderer(
        new IllegalStateException("rejected"), "decoder", 0, format, support, false, code);
  }

  private static ExoPlayerEventListener listener(VideoPlayerCallbacks callbacks) {
    return new ExoPlayerEventListener(mock(ExoPlayer.class), callbacks) {
      @Override
      protected void sendInitialized() {}
    };
  }
}
