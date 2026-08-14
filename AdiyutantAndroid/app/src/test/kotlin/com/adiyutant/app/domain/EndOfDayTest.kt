package com.adiyutant.app.domain

import com.adiyutant.app.domain.model.DayPhase
import com.adiyutant.app.domain.model.EndDayAction
import com.adiyutant.app.domain.model.Mood
import com.adiyutant.app.domain.model.PlannerEvent
import com.adiyutant.app.domain.model.SchedulePurpose
import com.adiyutant.app.domain.model.SessionKind
import com.adiyutant.app.domain.model.SheetState
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class EndOfDayTest {

    private val today = SEED_TODAY
    private val s4 = blockIdFor("t4", today)

    @Test
    fun keepTomorrowMovesBlockToDateOnlyTomorrow() {
        val (core, _) = plannerCoreAt(msAt(today, 18, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.EndDayTaskHandled(s4, EndDayAction.KEEP_TOMORROW, null))
        assertTrue(state.blocks.none { it.blockId == s4 })
        val task = state.tasks.single { it.taskId == "t4" }
        assertEquals(today.plusDays(1), task.plannedDate)
        assertEquals(false, task.postponed)
        assertTrue(dateOnlyTasksOn(state, today.plusDays(1)).any { it.taskId == "t4" })
    }

    @Test
    fun toBacklogClearsDateAndNoBlock() {
        val (core, _) = plannerCoreAt(msAt(today, 18, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.EndDayTaskHandled(s4, EndDayAction.TO_BACKLOG, null))
        assertTrue(state.blocks.none { it.blockId == s4 })
        val task = state.tasks.single { it.taskId == "t4" }
        assertNull(task.plannedDate)
        assertEquals(false, task.postponed)
        assertTrue(backlogGroups(state).noDate.any { it.taskId == "t4" })
    }

    @Test
    fun toDateSetsDateOnlyOnTargetDate() {
        val (core, _) = plannerCoreAt(msAt(today, 18, 0))
        var state = seedState(today)
        val target = today.plusDays(3)
        state = core.reduce(state, PlannerEvent.EndDayTaskHandled(s4, EndDayAction.TO_DATE, target))
        assertTrue(state.blocks.none { it.blockId == s4 })
        assertEquals(target, state.tasks.single { it.taskId == "t4" }.plannedDate)
        assertTrue(dateOnlyTasksOn(state, target).any { it.taskId == "t4" })
    }

    @Test
    fun chooseTimeKeepsBlockUntilConfirm() {
        val (core, _) = plannerCoreAt(msAt(today, 18, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.EndDayTaskHandled(s4, EndDayAction.CHOOSE_TIME, null))
        assertTrue(state.blocks.any { it.blockId == s4 })
        val sheet = state.sheet as SheetState.ScheduleSheet
        assertEquals(SchedulePurpose.END_DAY, sheet.request.purpose)
        assertEquals(today, sheet.request.sourceDate)
        assertEquals(today.plusDays(1), sheet.request.initialDate)
        assertEquals(false, sheet.request.lockDate)
    }

    @Test
    fun handlingBlockWithActiveSessionStopsItAsDayEnd() {
        val (core, clock) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.StartBlock(s4))
        clock.advance(90_000)
        state = core.reduce(state, PlannerEvent.EndDayTaskHandled(s4, EndDayAction.KEEP_TOMORROW, null))

        assertNull(state.activeSession)
        assertEquals(today.plusDays(1), state.tasks.single { it.taskId == "t4" }.plannedDate)
        assertTrue(state.blocks.none { it.blockId == s4 })
        val fact = state.factSessions.single()
        assertEquals(SessionKind.STOPPED_AT_DAY_END, fact.kind)
        assertEquals("t4", fact.taskId)
        assertEquals(s4, fact.blockId)
        assertEquals(90L, fact.durationSec)
    }

    @Test
    fun endDayFinishedClosesSessionAndCompletesDay() {
        val (core, clock) = plannerCoreAt(msAt(today, 18, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.CheckInSubmitted(2, 2, Mood.OK, null))
        state = core.reduce(state, PlannerEvent.StartBlock(s4))
        clock.advance(120_000)
        state = core.reduce(state, PlannerEvent.OpenEndOfDay)
        assertEquals(DayPhase.END_OF_DAY, state.dayPhase)
        assertEquals(SheetState.EndDaySheet, state.sheet)

        state = core.reduce(state, PlannerEvent.EndDayEnergySelected(1))
        state = core.reduce(state, PlannerEvent.EndDayFinished)

        assertEquals(DayPhase.DAY_COMPLETED, state.dayPhase)
        assertNull(state.activeSession)
        assertNull(state.sheet)
        assertEquals(1, state.endDayReview?.energy)
        val fact = state.factSessions.single()
        assertEquals(SessionKind.STOPPED_AT_DAY_END, fact.kind)
        assertEquals(120L, fact.durationSec)
    }

    @Test
    fun cancelEndDayReturnsToRunningLossless() {
        val (core, _) = plannerCoreAt(msAt(today, 18, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.CheckInSubmitted(2, 2, Mood.OK, null))
        val beforeTasks = state.tasks
        val beforeBlocks = state.blocks

        state = core.reduce(state, PlannerEvent.OpenEndOfDay)
        assertEquals(DayPhase.END_OF_DAY, state.dayPhase)
        state = core.reduce(state, PlannerEvent.CloseSheet)

        assertEquals(DayPhase.RUNNING, state.dayPhase)
        assertNull(state.sheet)
        assertEquals(beforeTasks, state.tasks)
        assertEquals(beforeBlocks, state.blocks)
        assertTrue(state.factSessions.isEmpty())
        assertNull(state.activeSession)
    }

    @Test
    fun upsertDoesNotDuplicateTaskIds() {
        val (core, _) = plannerCoreAt(msAt(today, 18, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.EndDayTaskHandled(s4, EndDayAction.TO_BACKLOG, null))
        state = core.reduce(state, PlannerEvent.EndDayTaskHandled(blockIdFor("t5", today), EndDayAction.KEEP_TOMORROW, null))
        assertEquals(1, state.tasks.count { it.taskId == "t4" })
        assertEquals(1, state.tasks.count { it.taskId == "t5" })
        assertEquals(22, state.tasks.size)
        assertEquals(0, state.blocks.count { it.taskId == "t4" })
        assertEquals(0, state.blocks.count { it.taskId == "t5" })
    }

    // ── W2: отмена ScheduleSheet (purpose=END_DAY) восстанавливает EndDaySheet ──

    @Test
    fun cancelEndDayScheduleRestoresEndDaySheetLossless() {
        val (core, _) = plannerCoreAt(msAt(today, 18, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.OpenEndOfDay)
        val beforeTasks = state.tasks
        val beforeBlocks = state.blocks

        state = core.reduce(state, PlannerEvent.EndDayTaskHandled(s4, EndDayAction.CHOOSE_TIME, null))
        assertTrue(state.sheet is SheetState.ScheduleSheet)

        state = core.reduce(state, PlannerEvent.ScheduleCancelled)

        assertEquals(SheetState.EndDaySheet, state.sheet)
        assertEquals(DayPhase.END_OF_DAY, state.dayPhase)
        assertEquals(beforeTasks, state.tasks)
        assertEquals(beforeBlocks, state.blocks)
        assertTrue(state.factSessions.isEmpty())
        assertNull(state.activeSession)
        assertNull(state.pendingDoElse)
    }

    @Test
    fun cancelEndDayScheduleKeepsActiveSessionAndSourceBlock() {
        val (core, clock) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.StartBlock(s4))
        clock.advance(90_000)
        state = core.reduce(state, PlannerEvent.OpenEndOfDay)
        state = core.reduce(state, PlannerEvent.EndDayTaskHandled(s4, EndDayAction.CHOOSE_TIME, null))
        val sessionBefore = state.activeSession
        val blockBefore = state.blocks.single { it.blockId == s4 }

        state = core.reduce(state, PlannerEvent.ScheduleCancelled)

        assertEquals(SheetState.EndDaySheet, state.sheet)
        assertEquals(DayPhase.END_OF_DAY, state.dayPhase)
        assertEquals(sessionBefore, state.activeSession)
        assertEquals(blockBefore, state.blocks.single { it.blockId == s4 })
        assertTrue(state.factSessions.isEmpty())
        assertNull(state.pendingDoElse)
    }

    @Test
    fun plainScheduleCancelClosesSheetWithoutEndDayRestore() {
        val (core, _) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.OpenTaskDetails(s4, today))
        state = core.reduce(state, PlannerEvent.TaskDetailsReschedule(sameDateOnly = true))
        assertTrue(state.sheet is SheetState.ScheduleSheet)

        state = core.reduce(state, PlannerEvent.ScheduleCancelled)

        assertNull(state.sheet)
        assertEquals(DayPhase.DAY_NOT_STARTED, state.dayPhase)
    }
}
