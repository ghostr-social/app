package social.ghostr

import android.content.Intent
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [35])
class IncomingVideoShareIntentCaptureTest {
    @Test
    fun createsAnIdentifierOnlyForShareIntents() {
        val capture = IncomingVideoShareIntentCapture()

        assertNotNull(capture.new(shareIntent()).captureId)
        assertNull(capture.new(Intent(Intent.ACTION_MAIN)).captureId)
    }
}
