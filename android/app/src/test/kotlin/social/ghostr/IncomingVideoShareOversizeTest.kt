package social.ghostr

import org.junit.Assert.assertThrows
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [35])
class IncomingVideoShareOversizeTest {
    @Test
    fun rejectsKnownContentLargerThanTheCopyLimit() {
        val context = shareTestContext(
            ShareTestProvider(byteArrayOf(1), "video/mp4", reportedSize = Long.MAX_VALUE),
        )
        val receiver = IncomingVideoShareReceiver(context)

        assertThrows(IncomingVideoShareException::class.java) {
            receiver.receive(capturedShareIntent(type = "video/mp4"))
        }
    }
}
