package social.ghostr

internal class AppUpdateFailure(
    val code: String,
    override val message: String,
) : Exception(message)
