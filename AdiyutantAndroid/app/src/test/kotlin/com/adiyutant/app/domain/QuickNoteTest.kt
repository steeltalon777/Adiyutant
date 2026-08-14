package com.adiyutant.app.domain

import com.adiyutant.app.domain.model.DoElseHandling
import com.adiyutant.app.domain.model.DoElseKind
import com.adiyutant.app.domain.model.DoElseSelection
import com.adiyutant.app.domain.model.NoteKind
import com.adiyutant.app.domain.model.PlannerEvent
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class QuickNoteTest {

    private val today = SEED_TODAY

    @Test
    fun blankTextIsNoOp() {
        val (core, _) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.QuickNoteAdded("   ", NoteKind.NOTE))
        state = core.reduce(state, PlannerEvent.QuickNoteAdded("", NoteKind.IDEA))
        assertTrue(state.quickNotesToday.isEmpty())
    }

    @Test
    fun noteIsInsertedAtFrontAndTrimmed() {
        val (core, clock) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.QuickNoteAdded("  первая  ", NoteKind.IDEA))
        clock.advance(1_000)
        state = core.reduce(state, PlannerEvent.QuickNoteAdded("вторая", NoteKind.NOTE))
        assertEquals(listOf("вторая", "первая"), state.quickNotesToday.map { it.text })
        assertEquals(NoteKind.IDEA, state.quickNotesToday.last().kind)
        assertTrue(state.quickNotesToday.all { it.text == it.text.trim() })
    }

    @Test
    fun capAtFiveNewestFirst() {
        val (core, clock) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        for (i in 1..7) {
            state = core.reduce(state, PlannerEvent.QuickNoteAdded("заметка $i", NoteKind.NOTE))
            clock.advance(1_000)
        }
        assertEquals(5, state.quickNotesToday.size)
        assertEquals("заметка 7", state.quickNotesToday.first().text)
        assertTrue(state.quickNotesToday.none { it.text == "заметка 1" })
        assertTrue(state.quickNotesToday.none { it.text == "заметка 2" })
    }

    @Test
    fun contextLabelFromActivePlannedBlock() {
        val (core, _) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.StartBlock(blockIdFor("t3", today)))
        state = core.reduce(state, PlannerEvent.QuickNoteAdded("заметка", NoteKind.SUMMARY))
        assertEquals("Deep work: React компоненты", state.quickNotesToday.single().contextLabel)
    }

    @Test
    fun contextLabelFromUnplannedSessionLabel() {
        val (core, _) = plannerCoreAt(msAt(today, 12, 0))
        var state = seedState(today)
        state = core.reduce(state, PlannerEvent.StartBlock(blockIdFor("t3", today)))
        state = core.reduce(state, PlannerEvent.DoElseRequested(blockIdFor("t3", today)))
        state = core.reduce(state, PlannerEvent.DoElseConfirmed(
            DoElseSelection(DoElseKind.BREAK, "Перерыв", null),
            DoElseHandling.LEAVE,
        ))
        state = core.reduce(state, PlannerEvent.QuickNoteAdded("заметка", NoteKind.NOTE))
        assertEquals("Перерыв", state.quickNotesToday.single().contextLabel)
    }

    @Test
    fun contextLabelNullWithoutSession() {
        val (core, _) = plannerCoreAt(msAt(today, 12, 0))
        val state = core.reduce(seedState(today), PlannerEvent.QuickNoteAdded("заметка", NoteKind.NOTE))
        assertNull(state.quickNotesToday.single().contextLabel)
    }
}
