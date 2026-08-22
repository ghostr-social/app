// Copyright 2013 The Flutter Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package io.flutter.plugins.videoplayer;

import static org.junit.Assert.assertEquals;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

import androidx.media3.common.Player;
import androidx.media3.exoplayer.ExoPlayer;
import java.util.ArrayList;
import java.util.List;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.robolectric.RobolectricTestRunner;

@RunWith(RobolectricTestRunner.class)
public final class ExoPlayerWarpFirstFrameTest {
  private static final String TOKEN = "AAAAAAAAAAAAAAAAAAAAAA";

  private static final class TestListener extends ExoPlayerEventListener {
    TestListener(
        ExoPlayer player, VideoPlayerCallbacks callbacks, WarpFirstFrameReporter reporter) {
      super(player, callbacks, reporter);
    }

    @Override
    protected void sendInitialized() {}
  }

  @Test
  public void readyIsNotRenderedAndRenderedPublishesExactlyOnce() {
    List<String> published = new ArrayList<>();
    ExoPlayer player = mock(ExoPlayer.class);
    when(player.getDuration()).thenReturn(10L);
    TestListener listener = listener(player, reporter(published));

    listener.onPlaybackStateChanged(Player.STATE_READY);
    assertEquals(List.of(), published);
    listener.onRenderedFirstFrame();
    listener.onRenderedFirstFrame();

    assertEquals(List.of(TOKEN), published);
  }

  @Test
  public void disposeBeforeRenderedFrameSuppressesPublication() {
    List<String> published = new ArrayList<>();
    TestListener listener = listener(mock(ExoPlayer.class), reporter(published));

    listener.dispose();
    listener.onRenderedFirstFrame();

    assertEquals(List.of(), published);
  }

  @Test
  public void absentAttemptTokenPreservesOrdinaryPlayerBehavior() {
    List<String> published = new ArrayList<>();
    WarpFirstFrameReporter reporter = WarpFirstFrameReporter.forAttempt(null, published::add);

    listener(mock(ExoPlayer.class), reporter).onRenderedFirstFrame();

    assertEquals(List.of(), published);
  }

  private static WarpFirstFrameReporter reporter(List<String> published) {
    return WarpFirstFrameReporter.forAttempt(TOKEN, published::add);
  }

  private static TestListener listener(ExoPlayer player, WarpFirstFrameReporter reporter) {
    return new TestListener(player, mock(VideoPlayerCallbacks.class), reporter);
  }
}
