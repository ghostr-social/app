package app.ghostr

import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [35])
class IncomingVideoShareMissingMetadataTest {
    @Test
    fun acceptsAProviderWithoutOpenableMetadata() {
        val provider = ShareTestProvider(
            byteArrayOf(1),
            "video/mp4",
            hasMetadata = false,
        )
        val receiver = IncomingVideoShareReceiver(shareTestContext(provider))

        val share = receiver.receive(capturedShareIntent(type = "video/mp4"))!!

        assertEquals("video", share.label)
        receiver.release(share.path)
    }
}
