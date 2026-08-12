package app.ghostr

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import java.io.File
import java.util.concurrent.TimeUnit

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [35])
class IncomingVideoShareBridgeRecreationTest {
    @Test
    fun recreatedBridgeRestoresTheCopyStartedByItsDisposedPredecessor() {
        val provider = BlockingShareTestProvider("video".toByteArray())
        val context = shareTestContext(provider)
        val captured = IncomingVideoShareIntentCapture { "capture" }.new(shareIntent())
        val first = IncomingVideoShareBridge(
            context,
            TestBinaryMessenger(),
            captured,
            1L,
        ) {}
        assertTrue(provider.openStarted.await(5, TimeUnit.SECONDS))
        first.dispose()
        val messenger = TestBinaryMessenger()
        val recreated = IncomingVideoShareBridge(context, messenger, captured, 1L) {}

        provider.allowOpen.countDown()
        val payload = messenger.invoke("takePendingVideo") as Map<*, *>
        val path = payload["path"] as String

        assertTrue(File(path).isFile)
        assertEquals(1, provider.openCount)
        messenger.invoke("releaseVideo", path)
        recreated.dispose()
    }
}
