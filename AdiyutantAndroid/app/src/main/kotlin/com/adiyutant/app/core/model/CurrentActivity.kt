package com.adiyutant.app.core.model

/**
 * What the user is (or should be) doing right now — mirrors
 * `CurrentActivityDto` from the Core facade.
 */
data class CurrentActivity(
    val activityKind: String,
    val activeTask: String,
    val activeBlock: String,
    val nextCheckpointAt: String,
    val recommendedPrompt: String,
)
