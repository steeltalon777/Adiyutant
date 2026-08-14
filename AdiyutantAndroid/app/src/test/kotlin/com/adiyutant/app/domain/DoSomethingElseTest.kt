package com.adiyutant.app.domain

import com.adiyutant.app.domain.model.ActiveKind
import com.adiyutant.app.domain.model.DayPhase
import com.adiyutant.app.domain.model.DoElseHandling
import com.adiyutant.app.domain.model.DoElseKind
import com.adiyutant.app.domain.model.DoElseSelection
import com.adiyutant.app.domain.model.PlannerEvent
import com.adiyutant.app.domain.model.PlannerState
import com.adiyutant.app.domain.model.ScheduleMode
import com.adiyutant.app.domain.model.SchedulePurpose
import com.adiyutant.app.domain.model.SessionKind
import com.adiyutant.app.domain.model.SheetState
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import java.time.LocalTime

class DoSomethingElseTest {

    private val today = SEED_TODAY
    private val s3 = blockIdFor("t3", today)
    private val s4 = blockIdFor("t4", today)

    private fun startT3(core: PlannerCore, state: PlannerState): PlannerState =
        core.reduce(state, PlannerEvent.StartBlock(s3))

    private fun selection(kind: DoElseKind, label: String = "", target: String? = null) =
        DoElseSelection(kind, label, target)

    @Test
    fun leaveStartsAlternativeSessionAndRecordsInterrupted() {
        val (core, clock) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = startT3(core, state)
        clock.advance(30_000)
        state = core.reduce(state, PlannerEvent.DoElseRequested(s3))
        assertEquals(SheetState.DoElseSheet(s3), state.sheet)

        state = core.reduce(state, PlannerEvent.DoElseConfirmed(selection(DoElseKind.UNPLANNED, "Перекусить"), DoElseHandling.LEAVE))

        val interrupted = state.factSessions.single()
        assertEquals(SessionKind.PLANNED_INTERRUPTED, interrupted.kind)
        assertEquals(30L, interrupted.durationSec)
        assertEquals("t3", interrupted.taskId)
        assertEquals(s3, interrupted.blockId)
        assertNull(interrupted.ease)
        assertEquals(45, state.blocks.single { it.taskId == "t3" }.progressPercent)

        val alt = state.activeSession!!
        assertEquals(ActiveKind.UNPLANNED_WORK, alt.kind)
        assertEquals("Перекусить", alt.label)
        assertEquals(SessionKind.UNPLANNED, alt.sessionKind)
        assertEquals(s3, alt.parentBlockId)
        assertEquals(DayPhase.RUNNING, state.dayPhase)
        assertNull(state.sheet)
    }

    @Test
    fun leaveWithoutActiveSessionStillStartsAlternative() {
        val (core, _) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.DoElseRequested(s3))
        state = core.reduce(state, PlannerEvent.DoElseConfirmed(selection(DoElseKind.BREAK, "Перерыв"), DoElseHandling.LEAVE))
        assertTrue(state.factSessions.isEmpty())
        assertEquals(ActiveKind.UNPLANNED_WORK, state.activeSession?.kind)
        assertEquals(s3, state.activeSession?.parentBlockId)
    }

    @Test
    fun shiftDefersToScheduleSheetAndKeepsSession() {
        val (core, clock) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = startT3(core, state)
        clock.advance(10_000)
        state = core.reduce(state, PlannerEvent.DoElseRequested(s3))
        state = core.reduce(state, PlannerEvent.DoElseConfirmed(selection(DoElseKind.BREAK, "Перерыв"), DoElseHandling.SHIFT))

        val pending = state.pendingDoElse!!
        assertEquals(s3, pending.blockId)
        assertEquals(DoElseKind.BREAK, pending.selection.kind)

        val sheet = state.sheet as SheetState.ScheduleSheet
        assertEquals(ScheduleMode.RESCHEDULE, sheet.request.mode)
        assertEquals(SchedulePurpose.DO_ELSE, sheet.request.purpose)
        assertEquals(today, sheet.request.sourceDate)
        assertEquals(today, sheet.request.initialDate)
        assertEquals(true, sheet.request.lockDate)

        assertTrue(state.activeSession != null)
        assertTrue(state.factSessions.isEmpty())
    }

    @Test
    fun moveUsesTomorrowWithoutLock() {
        val (core, _) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.DoElseRequested(s3))
        state = core.reduce(state, PlannerEvent.DoElseConfirmed(selection(DoElseKind.PERSONAL, "Личное"), DoElseHandling.MOVE))
        val sheet = state.sheet as SheetState.ScheduleSheet
        assertEquals(today.plusDays(1), sheet.request.initialDate)
        assertEquals(false, sheet.request.lockDate)
        assertEquals(SchedulePurpose.DO_ELSE, sheet.request.purpose)
        assertEquals(s3, state.pendingDoElse?.blockId)
    }

    @Test
    fun confirmAfterShiftMovesBlockAndStartsAlternative() {
        val (core, clock) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = startT3(core, state)
        clock.advance(10_000)
        state = core.reduce(state, PlannerEvent.DoElseRequested(s3))
        state = core.reduce(state, PlannerEvent.DoElseConfirmed(selection(DoElseKind.UNPLANNED, "Перекусить"), DoElseHandling.SHIFT))
        val sheet = state.sheet as SheetState.ScheduleSheet

        val duration = computeBlockDuration(state.blocks.single { it.taskId == "t3" })
        val slot = firstFreeSlot(state, today, duration, s3)
        clock.advance(20_000)
        state = core.reduce(state, PlannerEvent.ScheduleConfirmed(sheet.request, today, slot.first, slot.second))

        assertNull(state.pendingDoElse)
        assertNull(state.sheet)
        val interrupted = state.factSessions.single()
        assertEquals(SessionKind.PLANNED_INTERRUPTED, interrupted.kind)
        assertEquals(30L, interrupted.durationSec)
        val alt = state.activeSession!!
        assertEquals(ActiveKind.UNPLANNED_WORK, alt.kind)
        assertEquals("Перекусить", alt.label)
        assertEquals(s3, alt.parentBlockId)
    }

    @Test
    fun cancelDoElseKeepsOriginalSessionIntact() {
        val (core, clock) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = startT3(core, state)
        clock.advance(5_000)
        state = core.reduce(state, PlannerEvent.DoElseRequested(s3))
        state = core.reduce(state, PlannerEvent.DoElseConfirmed(selection(DoElseKind.UNPLANNED, "Перерыв"), DoElseHandling.MOVE))
        val beforeSession = state.activeSession
        val beforeBlocks = state.blocks

        state = core.reduce(state, PlannerEvent.ScheduleCancelled)
        assertNull(state.pendingDoElse)
        assertNull(state.sheet)
        assertEquals(beforeSession, state.activeSession)
        assertEquals(beforeBlocks, state.blocks)
        assertTrue(state.factSessions.isEmpty())
    }

    @Test
    fun otherTaskStartsSessionOnRealBlock() {
        val (core, _) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = startT3(core, state)
        state = core.reduce(state, PlannerEvent.DoElseRequested(s3))
        state = core.reduce(state, PlannerEvent.DoElseConfirmed(selection(DoElseKind.OTHER_TASK, "", s4), DoElseHandling.LEAVE))

        assertEquals(SessionKind.PLANNED_INTERRUPTED, state.factSessions.single().kind)
        val alt = state.activeSession!!
        assertEquals(ActiveKind.PLANNED, alt.kind)
        assertEquals("t4", alt.taskId)
        assertEquals(s4, alt.blockId)
        assertEquals(s3, alt.parentBlockId)
        assertEquals("Sprint Planning", alt.label)
        assertEquals(LocalTime.of(14, 0), alt.plannedStart)
        assertEquals(LocalTime.of(15, 0), alt.plannedEnd)
        assertNull(alt.sessionKind)
    }

    @Test
    fun finishingUnplannedDoesNotCompleteOriginalBlock() {
        val (core, clock) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = startT3(core, state)
        clock.advance(20_000)
        state = core.reduce(state, PlannerEvent.DoElseRequested(s3))
        state = core.reduce(state, PlannerEvent.DoElseConfirmed(selection(DoElseKind.BREAK, "Перерыв"), DoElseHandling.LEAVE))
        clock.advance(40_000)
        state = core.reduce(state, PlannerEvent.FinishActiveSession)

        val fact = state.factSessions.last()
        assertEquals(SessionKind.BREAK, fact.kind)
        assertEquals(40L, fact.durationSec)
        assertEquals("Перерыв", fact.label)
        assertNull(fact.taskId)
        assertNull(fact.blockId)
        assertEquals(s3, fact.parentBlockId)
        assertNull(state.activeSession)
        assertEquals(45, state.blocks.single { it.taskId == "t3" }.progressPercent)
    }

    @Test
    fun sessionKindMapsPerSelection() {
        val (core, _) = plannerCoreAt(msAt(today, 12, 0))
        val expected = mapOf(
            DoElseKind.UNPLANNED to SessionKind.UNPLANNED,
            DoElseKind.BREAK to SessionKind.BREAK,
            DoElseKind.PERSONAL to SessionKind.PERSONAL,
            DoElseKind.OTHER to SessionKind.OTHER,
        )
        for ((kind, sessionKind) in expected) {
            var state = seedState(today)
            state = core.reduce(state, PlannerEvent.StartBlock(s3))
            state = core.reduce(state, PlannerEvent.DoElseRequested(s3))
            state = core.reduce(state, PlannerEvent.DoElseConfirmed(selection(kind, "label"), DoElseHandling.LEAVE))
            assertEquals("mismatch for $kind", sessionKind, state.activeSession?.sessionKind)
        }
    }

    @Test
    fun defaultLabelUsedWhenSelectionLabelBlank() {
        val (core, _) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.StartBlock(s3))
        state = core.reduce(state, PlannerEvent.DoElseRequested(s3))
        state = core.reduce(state, PlannerEvent.DoElseConfirmed(selection(DoElseKind.BREAK), DoElseHandling.LEAVE))
        assertEquals("Перерыв", state.activeSession?.label)
    }
}
