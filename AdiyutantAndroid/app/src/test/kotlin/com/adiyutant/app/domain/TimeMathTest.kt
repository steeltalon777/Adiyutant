package com.adiyutant.app.domain

import com.adiyutant.app.domain.model.BlockAccent
import com.adiyutant.app.domain.model.ScheduledBlock
import org.junit.Assert.assertEquals
import org.junit.Test
import java.time.LocalDate
import java.time.LocalTime

class TimeMathTest {

    @Test
    fun minToTimeConvertsMinutes() {
        assertEquals(LocalTime.of(9, 0), minToTime(9 * 60))
        assertEquals(LocalTime.of(13, 45), minToTime(13 * 60 + 45))
        assertEquals(LocalTime.of(0, 0), minToTime(0))
    }

    @Test
    fun timeToMinConvertsBack() {
        assertEquals(540, timeToMin(LocalTime.of(9, 0)))
        assertEquals(825, timeToMin(LocalTime.of(13, 45)))
    }

    @Test
    fun addMinAddsMinutes() {
        assertEquals(LocalTime.of(9, 30), addMin(LocalTime.of(9, 0), 30))
        assertEquals(LocalTime.of(13, 0), addMin(LocalTime.of(11, 30), 90))
    }

    @Test
    fun computeBlockDurationIsEndMinusStart() {
        val block = ScheduledBlock(
            "b1", "t1", LocalDate.of(2026, 8, 14),
            LocalTime.of(11, 30), LocalTime.of(13, 0), 0, BlockAccent.BLUE,
        )
        assertEquals(90, computeBlockDuration(block))
    }

    @Test
    fun fmtMinsFormatsRanges() {
        assertEquals("~45м", fmtMins(45))
        assertEquals("~0м", fmtMins(0))
        assertEquals("1ч 30м", fmtMins(90))
        assertEquals("2ч", fmtMins(120))
        assertEquals("1ч", fmtMins(60))
    }

    @Test
    fun fmtMinsExactFormatsWithLeadingZero() {
        assertEquals("0ч 00м", fmtMinsExact(0))
        assertEquals("1ч 30м", fmtMinsExact(90))
        assertEquals("1ч 35м", fmtMinsExact(95))
        assertEquals("1ч 00м", fmtMinsExact(60))
    }

    @Test
    fun fmtElapsedSwitchesAtMinute() {
        assertEquals("0с", fmtElapsed(0))
        assertEquals("45с", fmtElapsed(45))
        assertEquals("59с", fmtElapsed(59))
        assertEquals("1м", fmtElapsed(60))
        assertEquals("1м", fmtElapsed(119))
    }

    @Test
    fun yyyyMMddIsIso() {
        assertEquals("2026-08-14", yyyyMMdd(LocalDate.of(2026, 8, 14)))
    }

    @Test
    fun russianArraysAreStatic() {
        assertEquals(listOf("Вс", "Пн", "Вт", "Ср", "Чт", "Пт", "Сб"), WEEKDAYS_SHORT)
        assertEquals(
            listOf("воскресенье", "понедельник", "вторник", "среда", "четверг", "пятница", "суббота"),
            WEEKDAYS_LONG,
        )
        assertEquals(
            listOf("янв", "фев", "мар", "апр", "мая", "июн", "июл", "авг", "сен", "окт", "ноя", "дек"),
            MONTHS,
        )
        assertEquals(
            listOf("января", "февраля", "марта", "апреля", "мая", "июня", "июля", "августа", "сентября", "октября", "ноября", "декабря"),
            MONTHS_LONG,
        )
    }

    @Test
    fun weekdayFunctionsUseRussianNames() {
        val friday = LocalDate.of(2026, 8, 14)
        assertEquals("Пт", weekdayShort(friday))
        assertEquals("пятница", weekdayLong(friday))
        assertEquals("Пн", weekdayShort(LocalDate.of(2026, 8, 17)))
        assertEquals("понедельник", weekdayLong(LocalDate.of(2026, 8, 17)))
        assertEquals("Вс", weekdayShort(LocalDate.of(2026, 8, 16)))
    }

    @Test
    fun formatDayMonthIsDayAndMonthWord() {
        assertEquals("14 августа", formatDayMonth(LocalDate.of(2026, 8, 14)))
        assertEquals("1 января", formatDayMonth(LocalDate.of(2026, 1, 1)))
    }
}
