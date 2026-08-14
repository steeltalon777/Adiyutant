package com.adiyutant.app.domain

import com.adiyutant.app.domain.model.ActiveKind
import com.adiyutant.app.domain.model.ActiveSession
import com.adiyutant.app.domain.model.Ease
import com.adiyutant.app.domain.model.PlannerEvent
import com.adiyutant.app.domain.model.SessionKind
import org.junit.Assert.assertEquals
import org.junit.Test

class ElapsedTimeTest {

    private val today = SEED_TODAY
    private val s3 = blockIdFor("t3", today)

    @Test
    fun elapsedIsNowMinusRunningSince() {
        val start = msAt(today, 12, 0)
        val session = ActiveSession(
            kind = ActiveKind.PLANNED,
            taskId = "t3",
            blockId = s3,
            parentBlockId = null,
            label = null,
            startedAtMs = start,
            runningSinceMs = start,
            accumulatedMs = 0,
            paused = false,
            plannedStart = null,
            plannedEnd = null,
        )
        assertEquals(0L, elapsedMs(session, start))
        assertEquals(5_000L, elapsedMs(session, start + 5_000))
    }

    @Test
    fun pauseFreezesElapsed() {
        val (core, clock) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.StartBlock(s3))
        clock.advance(5_000)
        state = core.reduce(state, PlannerEvent.PauseSession)
        assertEquals(5_000L, elapsedMs(state.activeSession!!, clock.nowMs()))
        clock.advance(10_000)
        assertEquals(5_000L, elapsedMs(state.activeSession!!, clock.nowMs()))
    }

    @Test
    fun pauseResumeSeriesPreservesTotalWithoutLossOrDoubling() {
        val start = msAt(today, 12, 0)
        val (core, clock) = plannerCoreAt(start)
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.StartBlock(s3))
        val originalStarted = state.activeSession!!.startedAtMs

        clock.advance(5_000)
        state = core.reduce(state, PlannerEvent.PauseSession)   // acc = 5000
        clock.advance(3_000)
        state = core.reduce(state, PlannerEvent.ResumeSession)  // runningSince = start + 8s
        clock.advance(5_000)
        state = core.reduce(state, PlannerEvent.PauseSession)   // acc = 10000
        clock.advance(2_000)
        state = core.reduce(state, PlannerEvent.ResumeSession)  // runningSince = start + 15s
        clock.advance(5_000)                                     // now = start + 20s

        assertEquals(15_000L, elapsedMs(state.activeSession!!, clock.nowMs()))
        assertEquals(10_000L, state.activeSession!!.accumulatedMs)
        assertEquals(originalStarted, state.activeSession!!.startedAtMs)
        assertEquals(start, state.activeSession!!.startedAtMs)
    }

    @Test
    fun startedAtMsNeverRewritten() {
        val start = msAt(today, 12, 0)
        val (core, clock) = plannerCoreAt(start)
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.StartBlock(s3))
        clock.advance(5_000)
        state = core.reduce(state, PlannerEvent.PauseSession)
        clock.advance(5_000)
        state = core.reduce(state, PlannerEvent.ResumeSession)
        clock.advance(5_000)
        assertEquals(start, state.activeSession!!.startedAtMs)
    }

    @Test
    fun durationSecIsFloorOfElapsedMillis() {
        val (core, clock) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.StartBlock(s3))
        clock.advance(30_900)
        state = core.reduce(state, PlannerEvent.FinishActiveSession)
        state = core.reduce(state, PlannerEvent.CompletionConfirmed(Ease.EASY))
        assertEquals(30L, state.factSessions.single().durationSec)
    }

    @Test
    fun startedAtMsCopiedToWorkSession() {
        val start = msAt(today, 12, 0)
        val (core, clock) = plannerCoreAt(start)
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.StartBlock(s3))
        clock.advance(60_000)
        state = core.reduce(state, PlannerEvent.FinishActiveSession)
        state = core.reduce(state, PlannerEvent.CompletionConfirmed(Ease.NORMAL))
        assertEquals(start, state.factSessions.single().startedAtMs)
    }

    @Test
    fun elapsedIgnoresNowWhenPaused() {
        val session = ActiveSession(
            kind = ActiveKind.UNPLANNED_WORK,
            taskId = null,
            blockId = null,
            parentBlockId = null,
            label = "Перерыв",
            startedAtMs = 1_000,
            runningSinceMs = null,
            accumulatedMs = 7_000,
            paused = true,
            plannedStart = null,
            plannedEnd = null,
            sessionKind = SessionKind.BREAK,
        )
        assertEquals(7_000L, elapsedMs(session, 1_000_000))
    }
}
