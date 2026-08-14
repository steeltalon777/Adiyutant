package com.adiyutant.app.ui.common

import com.adiyutant.app.domain.model.ScheduleMode
import com.adiyutant.app.ui.common.ScheduleSheetLogic.ConfirmLabel
import com.adiyutant.app.ui.common.ScheduleSheetLogic.DateChoiceKind
import java.time.LocalDate
import java.time.LocalTime
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Чистая JVM-логика ScheduleSheet (раздел 10 ТЗ): парсинг «HH:MM», чипы дат,
 * шаги времени, label confirm. Валидация времени/слотов покрыта в
 * ScheduleSheetContractTest (домен ScheduleValidator).
 */
class ScheduleSheetLogicTest {

    private val today: LocalDate = LocalDate.of(2026, 8, 14)

    // ── parseHhMm (10.3 п.6: формат HH:MM) ──

    @Test
    fun parse_валидные_форматы() {
        assertEquals(LocalTime.of(9, 0), ScheduleSheetLogic.parseHhMm("09:00"))
        assertEquals(LocalTime.of(9, 5), ScheduleSheetLogic.parseHhMm("9:05"))
        assertEquals(LocalTime.of(0, 0), ScheduleSheetLogic.parseHhMm("00:00"))
        assertEquals(LocalTime.of(23, 59), ScheduleSheetLogic.parseHhMm("23:59"))
        assertEquals(LocalTime.of(9, 0), ScheduleSheetLogic.parseHhMm(" 09:00 "))
    }

    @Test
    fun parse_невалидные_форматы_возвращают_null() {
        assertNull(ScheduleSheetLogic.parseHhMm(""))
        assertNull(ScheduleSheetLogic.parseHhMm("09:0"))
        assertNull(ScheduleSheetLogic.parseHhMm("9:5"))
        assertNull(ScheduleSheetLogic.parseHhMm("25:00"))
        assertNull(ScheduleSheetLogic.parseHhMm("12:60"))
        assertNull(ScheduleSheetLogic.parseHhMm("ab:cd"))
        assertNull(ScheduleSheetLogic.parseHhMm("09:00:30"))
        assertNull(ScheduleSheetLogic.parseHhMm("09-00"))
        assertNull(ScheduleSheetLogic.parseHhMm("900"))
    }

    // ── dateChoiceKind / dateChoices (10.4) ──

    @Test
    fun kind_по_датам() {
        assertEquals(DateChoiceKind.TODAY, ScheduleSheetLogic.dateChoiceKind(today, today))
        assertEquals(DateChoiceKind.TOMORROW, ScheduleSheetLogic.dateChoiceKind(today.plusDays(1), today))
        assertEquals(DateChoiceKind.OTHER, ScheduleSheetLogic.dateChoiceKind(today.plusDays(2), today))
        assertEquals(DateChoiceKind.OTHER, ScheduleSheetLogic.dateChoiceKind(today.minusDays(1), today))
    }

    @Test
    fun choices_sourceDate_первым_и_дедуп() {
        // sourceDate == today → [today, +1, +2]
        assertEquals(
            listOf(today, today.plusDays(1), today.plusDays(2)),
            ScheduleSheetLogic.dateChoices(today, today),
        )
        // sourceDate == tomorrow → [tomorrow, today, +2]
        assertEquals(
            listOf(today.plusDays(1), today, today.plusDays(2)),
            ScheduleSheetLogic.dateChoices(today.plusDays(1), today),
        )
        // sourceDate == today+2 → [today+2, today, +1] — дедуп +2
        assertEquals(
            listOf(today.plusDays(2), today, today.plusDays(1)),
            ScheduleSheetLogic.dateChoices(today.plusDays(2), today),
        )
        // sourceDate == вчера → [вчера, today, +1, +2]
        assertEquals(
            listOf(today.minusDays(1), today, today.plusDays(1), today.plusDays(2)),
            ScheduleSheetLogic.dateChoices(today.minusDays(1), today),
        )
    }

    // ── confirmLabel (10.3 п.8) ──

    @Test
    fun confirmLabel_по_режиму_и_дате() {
        assertEquals(
            ConfirmLabel.SCHEDULE,
            ScheduleSheetLogic.confirmLabel(ScheduleMode.SCHEDULE, today, today),
        )
        assertEquals(
            ConfirmLabel.SAVE,
            ScheduleSheetLogic.confirmLabel(ScheduleMode.RESCHEDULE, today, today),
        )
        assertEquals(
            ConfirmLabel.MOVE,
            ScheduleSheetLogic.confirmLabel(ScheduleMode.RESCHEDULE, today.plusDays(1), today),
        )
    }

    // ── Время вручную (10.4) ──

    @Test
    fun первое_нажатие_при_пустом_времени_09_00_плюс_длительность() {
        assertEquals(LocalTime.of(9, 0) to LocalTime.of(9, 45), ScheduleSheetLogic.initialFromEmpty(45))
        assertEquals(LocalTime.of(9, 0) to LocalTime.of(10, 30), ScheduleSheetLogic.initialFromEmpty(90))
        assertEquals(LocalTime.of(9, 0) to LocalTime.of(9, 15), ScheduleSheetLogic.initialFromEmpty(15))
    }

    @Test
    fun сдвиг_пересчитывает_end_от_нового_start() {
        assertEquals(LocalTime.of(9, 15) to LocalTime.of(10, 0), ScheduleSheetLogic.shifted(LocalTime.of(9, 0), 45, 15))
        assertEquals(LocalTime.of(8, 45) to LocalTime.of(9, 30), ScheduleSheetLogic.shifted(LocalTime.of(9, 0), 45, -15))
        assertEquals(LocalTime.of(10, 0) to LocalTime.of(11, 30), ScheduleSheetLogic.shifted(LocalTime.of(9, 0), 90, 60))
    }

    @Test
    fun end_редактируется_только_если_больше_start() {
        assertFalse(ScheduleSheetLogic.canSetEnd(null, LocalTime.of(10, 0)))
        assertFalse(ScheduleSheetLogic.canSetEnd(LocalTime.of(9, 0), null))
        assertFalse(ScheduleSheetLogic.canSetEnd(null, null))
        assertFalse(ScheduleSheetLogic.canSetEnd(LocalTime.of(9, 0), LocalTime.of(9, 0)))
        assertFalse(ScheduleSheetLogic.canSetEnd(LocalTime.of(9, 0), LocalTime.of(8, 0)))
        assertTrue(ScheduleSheetLogic.canSetEnd(LocalTime.of(9, 0), LocalTime.of(10, 0)))
    }

    @Test
    fun end_для_нового_start_равен_start_плюс_длительность() {
        assertEquals(LocalTime.of(9, 45), ScheduleSheetLogic.endForStart(LocalTime.of(9, 0), 45))
        assertEquals(LocalTime.of(10, 30), ScheduleSheetLogic.endForStart(LocalTime.of(9, 0), 90))
    }
}
