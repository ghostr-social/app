package app.ghostr

import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import java.io.File

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [35])
class IncomingVideoShareUnknownSizeTest {
    @Test
    fun boundsAndCopiesAProviderThatCannotReportItsSize() {
        val provider = ShareTestProvider(
            byteArrayOf(1),
            "video/mp4",
            reportedSize = null,
        )
        val receiver = IncomingVideoShareReceiver(shareTestContext(provider))

        val share = receiver.receive(capturedShareIntent(type = "video/mp4"))!!

        assertTrue(File(share.path).isFile)
        receiver.release(share.path)
    }
}
