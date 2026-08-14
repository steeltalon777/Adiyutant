package com.adiyutant.app.domain

import com.adiyutant.app.domain.model.ActiveKind
import com.adiyutant.app.domain.model.DayMode
import com.adiyutant.app.domain.model.DayPhase
import com.adiyutant.app.domain.model.DoElseHandling
import com.adiyutant.app.domain.model.DoElseKind
import com.adiyutant.app.domain.model.DoElseSelection
import com.adiyutant.app.domain.model.Ease
import com.adiyutant.app.domain.model.HeroState
import com.adiyutant.app.domain.model.Mood
import com.adiyutant.app.domain.model.NotDoneAction
import com.adiyutant.app.domain.model.PlannerEvent
import com.adiyutant.app.domain.model.ScheduleMode
import com.adiyutant.app.domain.model.SchedulePurpose
import com.adiyutant.app.domain.model.SessionKind
import com.adiyutant.app.domain.model.SheetState
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import java.time.LocalTime

class PlannerCoreStateMachineTest {

    private val today = SEED_TODAY
    private val s3 = blockIdFor("t3", today)

    @Test
    fun dayStartsNotStartedAndOnlyCheckInStartsIt() {
        val (core, _) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        assertEquals(DayPhase.DAY_NOT_STARTED, state.dayPhase)

        state = core.reduce(state, PlannerEvent.StartBlock(s3))
        assertEquals(DayPhase.DAY_NOT_STARTED, state.dayPhase)

        state = core.reduce(state, PlannerEvent.CheckInSubmitted(2, 2, Mood.OK, null))
        assertEquals(DayPhase.RUNNING, state.dayPhase)
        assertEquals(2, state.dayEnergy)
        assertEquals(2, state.dayCheckIn?.energy)
    }

    @Test
    fun readyDerivedFromRealTimeInsideWindow() {
        val now = msAt(today, 12, 0)
        val (core, clock) = plannerCoreAt(now)
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.CheckInSubmitted(2, 2, Mood.OK, null))
        assertEquals(HeroState.READY, deriveHeroState(state, clock.nowMs()))
    }

    @Test
    fun upcomingDerivedFromFutureBlock() {
        val now = msAt(today, 8, 0)
        val (core, clock) = plannerCoreAt(now)
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.CheckInSubmitted(2, 2, Mood.OK, null))
        assertEquals(HeroState.UPCOMING, deriveHeroState(state, clock.nowMs()))
    }

    @Test
    fun betweenBlocksAfterAllUnfinishedStarts() {
        val now = msAt(today, 18, 0)
        val (core, clock) = plannerCoreAt(now)
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.CheckInSubmitted(2, 2, Mood.OK, null))
        assertEquals(HeroState.BETWEEN_BLOCKS, deriveHeroState(state, clock.nowMs()))
    }

    @Test
    fun startBlockMakesHeroActive() {
        val now = msAt(today, 12, 0)
        val (core, clock) = plannerCoreAt(now)
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.StartBlock(s3))
        assertEquals(HeroState.ACTIVE, deriveHeroState(state, clock.nowMs()))
        val s = state.activeSession!!
        assertEquals(ActiveKind.PLANNED, s.kind)
        assertEquals("t3", s.taskId)
        assertEquals(s3, s.blockId)
        assertEquals(LocalTime.of(11, 30), s.plannedStart)
        assertEquals(LocalTime.of(13, 0), s.plannedEnd)
        assertEquals(now, s.startedAtMs)
        assertEquals(now, s.runningSinceMs)
        assertNull(s.sessionKind)
    }

    @Test
    fun pauseFreezesElapsedAndResumeContinues() {
        val start = msAt(today, 12, 0)
        val (core, clock) = plannerCoreAt(start)
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.StartBlock(s3))

        clock.advance(5_000)
        state = core.reduce(state, PlannerEvent.PauseSession)
        assertEquals(true, state.activeSession?.paused)
        assertEquals(5_000L, state.activeSession?.accumulatedMs)
        assertEquals(HeroState.PAUSED, deriveHeroState(state, clock.nowMs()))
        assertEquals(5_000L, elapsedMs(state.activeSession!!, clock.nowMs()))

        clock.advance(5_000)
        state = core.reduce(state, PlannerEvent.ResumeSession)
        assertEquals(false, state.activeSession?.paused)
        assertEquals(start, state.activeSession?.startedAtMs)

        clock.advance(5_000)
        assertEquals(10_000L, elapsedMs(state.activeSession!!, clock.nowMs()))
        assertEquals(HeroState.ACTIVE, deriveHeroState(state, clock.nowMs()))
    }

    @Test
    fun startBlockIsNoOpWhenSessionActive() {
        val (core, _) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.StartBlock(s3))
        val first = state.activeSession
        state = core.reduce(state, PlannerEvent.StartBlock(blockIdFor("t4", today)))
        assertEquals(first, state.activeSession)
    }

    @Test
    fun startBlockIsNoOpWhenBlockCompleted() {
        val (core, _) = plannerCoreAt(msAt(today, 12, 0))
        val state = core.reduce(seedState(today), PlannerEvent.StartBlock(blockIdFor("t1", today)))
        assertNull(state.activeSession)
    }

    @Test
    fun startBlockIsNoOpWhenBlockMissing() {
        val (core, _) = plannerCoreAt(msAt(today, 12, 0))
        val state = core.reduce(seedState(today), PlannerEvent.StartBlock("missing_block"))
        assertNull(state.activeSession)
    }

    @Test
    fun topThreeStartSharesStartRules() {
        val (core, _) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.TopThreeStart(s3))
        assertEquals("t3", state.activeSession?.taskId)
    }

    @Test
    fun finishPlannedSessionOpensBlockEndedSheet() {
        val now = msAt(today, 12, 0)
        val (core, clock) = plannerCoreAt(now)
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.StartBlock(s3))
        clock.advance(60_000)
        state = core.reduce(state, PlannerEvent.FinishActiveSession)
        assertEquals(SheetState.BlockEndedSheet(s3), state.sheet)
        assertTrue(state.activeSession != null)
        assertEquals(HeroState.BLOCK_ENDED_UNRESOLVED, deriveHeroState(state, clock.nowMs()))
        assertTrue(state.factSessions.isEmpty())
    }

    @Test
    fun stillWorkingClosesSheetAndKeepsSession() {
        val now = msAt(today, 12, 0)
        val (core, clock) = plannerCoreAt(now)
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.StartBlock(s3))
        clock.advance(60_000)
        state = core.reduce(state, PlannerEvent.FinishActiveSession)
        state = core.reduce(state, PlannerEvent.BlockEndedStillWorking)
        assertNull(state.sheet)
        assertTrue(state.activeSession != null)
        assertEquals(HeroState.ACTIVE, deriveHeroState(state, clock.nowMs()))
        assertEquals(60L, elapsedMs(state.activeSession!!, clock.nowMs()) / 1000)
    }

    @Test
    fun notDoneClearsSessionWithoutFact() {
        val now = msAt(today, 12, 0)
        val (core, clock) = plannerCoreAt(now)
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.StartBlock(s3))
        clock.advance(60_000)
        state = core.reduce(state, PlannerEvent.FinishActiveSession)
        state = core.reduce(state, PlannerEvent.BlockEndedNotDone)
        assertNull(state.activeSession)
        assertTrue(state.factSessions.isEmpty())
        assertEquals(SheetState.NotDoneSheet(s3), state.sheet)
    }

    @Test
    fun completionConfirmedMarksBlockAndRecordsFact() {
        val now = msAt(today, 12, 0)
        val (core, clock) = plannerCoreAt(now)
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.StartBlock(s3))
        clock.advance(30_000)
        state = core.reduce(state, PlannerEvent.FinishActiveSession)
        state = core.reduce(state, PlannerEvent.CompletionConfirmed(Ease.NORMAL))
        assertEquals(100, state.blocks.first { it.taskId == "t3" }.progressPercent)
        assertNull(state.activeSession)
        assertNull(state.sheet)
        val fact = state.factSessions.single()
        assertEquals(SessionKind.PLANNED_COMPLETED, fact.kind)
        assertEquals(Ease.NORMAL, fact.ease)
        assertEquals(30L, fact.durationSec)
        assertEquals(now, fact.startedAtMs)
    }

    @Test
    fun completionConfirmedIsNoOpWithoutBlockEndedSheet() {
        val (core, _) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.StartBlock(s3))
        state = core.reduce(state, PlannerEvent.CompletionConfirmed(Ease.EASY))
        assertTrue(state.activeSession != null)
        assertTrue(state.factSessions.isEmpty())
        assertEquals(45, state.blocks.first { it.taskId == "t3" }.progressPercent)
    }

    // ── W1: OpenCompletionSheet ──

    @Test
    fun openCompletionSheetOpensForValidUnfinishedBlock() {
        val (core, clock) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.StartBlock(s3))
        clock.advance(30_000)
        val beforeTasks = state.tasks
        val beforeBlocks = state.blocks
        val beforeSession = state.activeSession

        state = core.reduce(state, PlannerEvent.OpenCompletionSheet(s3))

        assertEquals(SheetState.CompletionSheet(s3), state.sheet)
        assertEquals(beforeTasks, state.tasks)
        assertEquals(beforeBlocks, state.blocks)
        assertEquals(beforeSession, state.activeSession)
        assertTrue(state.factSessions.isEmpty())
    }

    @Test
    fun openCompletionSheetIsNoOpForUnknownBlock() {
        val (core, _) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.OpenCompletionSheet("missing_block"))
        assertNull(state.sheet)
    }

    @Test
    fun openCompletionSheetIsNoOpForCompletedBlock() {
        val (core, _) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.OpenCompletionSheet(blockIdFor("t1", today)))
        assertNull(state.sheet)
        assertEquals(100, state.blocks.first { it.taskId == "t1" }.progressPercent)
    }

    @Test
    fun openCompletionSheetIsNoOpForIncompleteBlockWithoutSession() {
        val (core, _) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        // t3: progress=45 (незавершён), активной сессии нет
        state = core.reduce(state, PlannerEvent.OpenCompletionSheet(s3))
        assertNull(state.sheet)
        assertNull(state.activeSession)
        assertEquals(45, state.blocks.first { it.taskId == "t3" }.progressPercent)
    }

    @Test
    fun openCompletionSheetIsNoOpForIncompleteBlockWithOtherActiveSession() {
        val (core, _) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        // активна PLANNED-сессия блока t3, а лист просят для незавершённого t4
        state = core.reduce(state, PlannerEvent.StartBlock(s3))
        state = core.reduce(state, PlannerEvent.OpenCompletionSheet(blockIdFor("t4", today)))
        assertNull(state.sheet)
        assertEquals("t3", state.activeSession?.taskId)
        assertEquals(0, state.blocks.first { it.taskId == "t4" }.progressPercent)
    }

    @Test
    fun completionConfirmedWorksFromCompletionSheet() {
        val (core, clock) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.StartBlock(s3))
        clock.advance(30_000)
        state = core.reduce(state, PlannerEvent.OpenCompletionSheet(s3))
        assertEquals(SheetState.CompletionSheet(s3), state.sheet)

        state = core.reduce(state, PlannerEvent.CompletionConfirmed(Ease.EASY))

        assertEquals(100, state.blocks.first { it.taskId == "t3" }.progressPercent)
        assertNull(state.activeSession)
        assertNull(state.sheet)
        val fact = state.factSessions.single()
        assertEquals(SessionKind.PLANNED_COMPLETED, fact.kind)
        assertEquals(Ease.EASY, fact.ease)
        assertEquals(30L, fact.durationSec)
    }

    @Test
    fun taskDetailsCompleteIsNoOpForActiveBlock() {
        val (core, _) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.StartBlock(s3))
        state = core.reduce(state, PlannerEvent.TaskDetailsComplete(s3))
        assertEquals(45, state.blocks.first { it.taskId == "t3" }.progressPercent)
        assertTrue(state.activeSession != null)
    }

    @Test
    fun taskDetailsCompleteWorksWithoutActiveSession() {
        val (core, _) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.OpenTaskDetails(s3, today))
        state = core.reduce(state, PlannerEvent.TaskDetailsComplete(s3))
        assertEquals(100, state.blocks.first { it.taskId == "t3" }.progressPercent)
        assertNull(state.sheet)
        assertTrue(state.factSessions.isEmpty())
    }

    @Test
    fun notDoneMoveOpensRescheduleSheet() {
        val (core, clock) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.StartBlock(s3))
        clock.advance(60_000)
        state = core.reduce(state, PlannerEvent.FinishActiveSession)
        state = core.reduce(state, PlannerEvent.BlockEndedNotDone)
        state = core.reduce(state, PlannerEvent.NotDoneAction(NotDoneAction.MOVE))
        val sheet = state.sheet as SheetState.ScheduleSheet
        assertEquals(ScheduleMode.RESCHEDULE, sheet.request.mode)
        assertEquals(SchedulePurpose.PLAIN, sheet.request.purpose)
        assertEquals(today, sheet.request.sourceDate)
        assertEquals(today, sheet.request.initialDate)
        assertEquals(false, sheet.request.lockDate)
    }

    @Test
    fun notDoneToBacklogRemovesBlockToNoDate() {
        val (core, clock) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.StartBlock(s3))
        clock.advance(60_000)
        state = core.reduce(state, PlannerEvent.FinishActiveSession)
        state = core.reduce(state, PlannerEvent.BlockEndedNotDone)
        state = core.reduce(state, PlannerEvent.NotDoneAction(NotDoneAction.TO_BACKLOG))
        assertNull(state.sheet)
        assertTrue(state.blocks.none { it.taskId == "t3" })
        assertNull(state.tasks.single { it.taskId == "t3" }.plannedDate)
        assertTrue(backlogGroups(state).noDate.any { it.taskId == "t3" })
    }

    @Test
    fun notDoneLeaveJustClosesSheet() {
        val (core, clock) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.StartBlock(s3))
        clock.advance(60_000)
        state = core.reduce(state, PlannerEvent.FinishActiveSession)
        state = core.reduce(state, PlannerEvent.BlockEndedNotDone)
        val before = state.blocks
        state = core.reduce(state, PlannerEvent.NotDoneAction(NotDoneAction.LEAVE))
        assertNull(state.sheet)
        assertEquals(before, state.blocks)
    }

    @Test
    fun topThreeCompleteMarksBlockWithoutFact() {
        val (core, _) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.TopThreeComplete(s3))
        assertEquals(100, state.blocks.first { it.taskId == "t3" }.progressPercent)
        assertTrue(state.factSessions.isEmpty())
    }

    @Test
    fun simpleEventsUpdateDayState() {
        val (core, _) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.DayModeSelected(DayMode.FOCUS))
        assertEquals(DayMode.FOCUS, state.dayMode)
    }
}
