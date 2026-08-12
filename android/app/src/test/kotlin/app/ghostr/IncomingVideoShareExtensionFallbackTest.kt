package app.ghostr

import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [35])
class IncomingVideoShareExtensionFallbackTest {
    @Test
    fun resolvesGenericVideoFromSafeFileExtension() {
        val context = shareTestContext(
            ShareTestProvider(byteArrayOf(1), null, "folder\\shared.webm"),
        )
        val receiver = IncomingVideoShareReceiver(context)

        val share = receiver.receive(capturedShareIntent())!!

        assertEquals("video/webm", share.mimeType)
        assertEquals("shared.webm", share.label)
        receiver.release(share.path)
    }
}
