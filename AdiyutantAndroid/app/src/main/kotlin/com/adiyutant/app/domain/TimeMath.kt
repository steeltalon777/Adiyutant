package com.adiyutant.app.domain

import com.adiyutant.app.domain.model.ScheduledBlock
import java.time.LocalDate
import java.time.LocalTime
import java.time.format.DateTimeFormatter

/**
 * Форматирование времени/дат, точные формулы из App.jsx 79–99.
 * Русские массивы — статические (не локали устройства), для детерминизма.
 */
val WEEKDAYS_SHORT: List<String> = listOf("Вс", "Пн", "Вт", "Ср", "Чт", "Пт", "Сб")
val WEEKDAYS_LONG: List<String> = listOf(
    "воскресенье", "понедельник", "вторник", "среда", "четверг", "пятница", "суббота",
)
val MONTHS: List<String> = listOf("янв", "фев", "мар", "апр", "мая", "июн", "июл", "авг", "сен", "окт", "ноя", "дек")
val MONTHS_LONG: List<String> = listOf(
    "января", "февраля", "марта", "апреля", "мая", "июня",
    "июля", "августа", "сентября", "октября", "ноября", "декабря",
)

private val ISO_DATE: DateTimeFormatter = DateTimeFormatter.ofPattern("yyyy-MM-dd")

fun minToTime(min: Int): LocalTime = LocalTime.of(min / 60, min % 60)

fun timeToMin(t: LocalTime): Int = t.hour * 60 + t.minute

fun addMin(t: LocalTime, delta: Int): LocalTime = t.plusMinutes(delta.toLong())

fun computeBlockDuration(block: ScheduledBlock): Int = timeToMin(block.end) - timeToMin(block.start)

/** "~45м"; "1ч 30м"; "2ч". */
fun fmtMins(min: Int): String {
    if (min < 60) return "~${min}м"
    val h = min / 60
    val r = min % 60
    return if (r != 0) "${h}ч ${r}м" else "${h}ч"
}

/** "1ч 05м" — всегда часы и ведущий ноль минут. */
fun fmtMinsExact(min: Int): String {
    val h = min / 60
    val r = min % 60
    return "${h}ч ${r.toString().padStart(2, '0')}м"
}

/** <60с → "45с", иначе "Nм" (floor). */
fun fmtElapsed(sec: Long): String {
    if (sec < 60) return "${sec}с"
    return "${sec / 60}м"
}

/** ISO-дата для детерминированного blockId. */
fun yyyyMMdd(date: LocalDate): String = ISO_DATE.format(date)

/** «пятница», «понедельник» и т.д. */
fun weekdayLong(date: LocalDate): String = WEEKDAYS_LONG[date.dayOfWeek.value % 7]

/** «Пт», «Пн» и т.д. */
fun weekdayShort(date: LocalDate): String = WEEKDAYS_SHORT[date.dayOfWeek.value % 7]

/** "14 августа". */
fun formatDayMonth(date: LocalDate): String = "${date.dayOfMonth} ${MONTHS_LONG[date.monthValue - 1]}"
