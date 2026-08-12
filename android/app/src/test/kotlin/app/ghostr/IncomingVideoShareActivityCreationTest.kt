package app.ghostr

import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [35])
class IncomingVideoShareActivityCreationTest {
    @Test
    fun restoredShareKeepsItsIdentityAcrossActivityAndEngineRecreation() {
        val original = IncomingVideoShareActivityLifecycle(
            shareIntent(),
            null,
            IncomingVideoShareIntentCapture { "capture" },
        )
        val firstEngine = original.configureEngine()
        val recreated = IncomingVideoShareActivityLifecycle(
            shareIntent(),
            original.savedCaptureId,
            IncomingVideoShareIntentCapture { "unused" },
        )

        val recreatedEngine = recreated.configureEngine()
        val replacementEngine = recreated.configureEngine()

        assertEquals("capture", firstEngine.intent.savedId)
        assertEquals("capture", recreatedEngine.intent.savedId)
        assertEquals("capture", replacementEngine.intent.savedId)
        assertEquals(1L, firstEngine.generation)
        assertEquals(1L, recreatedEngine.generation)
        assertEquals(2L, replacementEngine.generation)
        assertEquals("capture", recreated.savedCaptureId)
    }
}
