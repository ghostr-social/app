package social.ghostr

import android.content.Intent
import org.junit.Assert.assertNull
import org.junit.Assert.assertThrows
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.RuntimeEnvironment
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [35])
class IncomingVideoShareMalformedTest {
    @Test
    fun ignoresNonSharesAndRejectsMissingIdentityOrUnsupportedVideo() {
        val context = RuntimeEnvironment.getApplication()
        val receiver = IncomingVideoShareReceiver(context)
        val launcher = CapturedIncomingVideoIntent(Intent(Intent.ACTION_MAIN), null)
        assertNull(receiver.receive(launcher))
        assertThrows(IncomingVideoShareException::class.java) {
            receiver.receive(CapturedIncomingVideoIntent(shareIntent(), null))
        }
        val provider = ShareTestProvider(byteArrayOf(1), null, "shared.bin")
        val providerReceiver = IncomingVideoShareReceiver(shareTestContext(provider))
        assertThrows(IncomingVideoShareException::class.java) {
            providerReceiver.receive(capturedShareIntent())
        }
    }
}
