package app.ghostr

import android.net.Uri
import org.junit.Assert.assertThrows
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.RuntimeEnvironment
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [35])
class IncomingVideoShareInvalidUriTest {
    @Test
    fun rejectsAFileUriBeforeReadingIt() {
        val context = RuntimeEnvironment.getApplication()
        val receiver = IncomingVideoShareReceiver(context)

        assertThrows(IncomingVideoShareException::class.java) {
            receiver.receive(capturedShareIntent(uri = Uri.parse("file:///tmp/video.mp4")))
        }
    }
}
