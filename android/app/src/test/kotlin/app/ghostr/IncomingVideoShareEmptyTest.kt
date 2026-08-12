package app.ghostr

import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [35])
class IncomingVideoShareEmptyTest {
    @Test
    fun rejectsEmptyContentWithoutLeavingAPartialCopy() {
        val context = shareTestContext(
            ShareTestProvider(byteArrayOf(), "video/mp4", reportedSize = -1L),
        )
        val before = incomingFiles(context).size
        val receiver = IncomingVideoShareReceiver(context)

        assertThrows(IncomingVideoShareException::class.java) {
            receiver.receive(capturedShareIntent(type = "video/mp4"))
        }
        assertEquals(before, incomingFiles(context).size)
    }

    private fun incomingFiles(context: android.content.Context): List<java.io.File> {
        return context.cacheDir.listFiles()
            ?.filter { it.name.startsWith("incoming-video-") }
            .orEmpty()
    }
}
