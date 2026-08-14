package com.adiyutant.app.core.fake

import com.adiyutant.app.core.CorePort
import com.adiyutant.app.core.model.CurrentActivity
import com.adiyutant.app.core.model.TodaySnapshot

/**
 * First [CorePort] implementation backed by sample data (ADR-0002).
 *
 * Purpose: let the UI shell build, navigate and be inspected without any
 * Rust dependency. Replaced by the real integration behind the same
 * interface once the spike in ADR-0003 concludes.
 */
class FakeCorePort : CorePort {

    override suspend fun getTodaySnapshot(): TodaySnapshot = TodaySnapshot(
        date = SampleData.today,
        mode = "focus",
        energy = "6",
        mood = "8",
        checkInCount = 2,
        hasMorningCheckIn = true,
        hasEveningCheckIn = false,
        habitCount = 3,
        habitEventsDone = 1,
        hasPlan = true,
        planTitle = "Plan for 2026-06-22",
        planItemCount = 3,
    )

    override suspend fun getCurrentActivity(): CurrentActivity = CurrentActivity(
        activityKind = "scheduled_task",
        activeTask = "Глубокая работа",
        activeBlock = "Глубокий блок",
        nextCheckpointAt = "",
        recommendedPrompt = "Ready to start: Глубокая работа",
    )
}
