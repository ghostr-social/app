package social.ghostr

import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [35])
class IncomingVideoShareFallbackLabelTest {
    @Test
    fun replacesAnUnsafeBlankLikeLabelWithASafeVideoName() {
        val context = shareTestContext(
            ShareTestProvider(byteArrayOf(1), "video/mp4", ".."),
        )
        val receiver = IncomingVideoShareReceiver(context)

        val share = receiver.receive(capturedShareIntent(type = "video/mp4"))!!

        assertEquals("shared-video.mp4", share.label)
        receiver.release(share.path)
    }
}
