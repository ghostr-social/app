package app.ghostr

import org.junit.Assert.assertArrayEquals
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import java.io.File

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [28])
class IncomingVideoShareLegacyTest {
    @Test
    fun readsTheGrantedStreamOnPreTiramisuAndroid() {
        val bytes = "legacy-video".toByteArray()
        val context = shareTestContext(ShareTestProvider(bytes, "video/mp4"))
        val receiver = IncomingVideoShareReceiver(context)

        val share = receiver.receive(capturedShareIntent())!!

        assertArrayEquals(bytes, File(share.path).readBytes())
        receiver.release(share.path)
    }
}
