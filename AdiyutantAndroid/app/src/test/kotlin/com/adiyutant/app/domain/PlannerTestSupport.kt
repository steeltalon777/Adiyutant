package com.adiyutant.app.domain

import com.adiyutant.app.data.SampleSeedData
import com.adiyutant.app.domain.model.CheckIn
import com.adiyutant.app.domain.model.DayMode
import com.adiyutant.app.domain.model.DayPhase
import com.adiyutant.app.domain.model.PlanMode
import com.adiyutant.app.domain.model.PlanUiState
import com.adiyutant.app.domain.model.PlannerState
import com.adiyutant.app.domain.model.ScheduledBlock
import com.adiyutant.app.domain.model.Task
import java.time.LocalDate
import java.time.LocalTime
import java.time.YearMonth
import java.time.ZoneId

/**
 * Детерминированные часы с управляемыми nowMs/today для чистых JUnit-тестов.
 */
class FakeClock(nowMs: Long, today: LocalDate) : Clock {
    private var currentNowMs = nowMs
    private var currentToday = today

    override fun nowMs(): Long = currentNowMs
    override fun today(): LocalDate = currentToday

    fun setNow(value: Long) {
        currentNowMs = value
    }

    fun advance(deltaMs: Long) {
        currentNowMs += deltaMs
    }

    fun setToday(value: LocalDate) {
        currentToday = value
    }
}

/** Детерминированная "сегодня" дата seed (как в JSX). */
val SEED_TODAY: LocalDate = LocalDate.of(2026, 8, 14)

private val TEST_ZONE: ZoneId = ZoneId.systemDefault()

/** Epoch-millis для [date] в [hour]:[min] по системной зоне. */
fun msAt(date: LocalDate, hour: Int, min: Int): Long =
    date.atTime(hour, min).atZone(TEST_ZONE).toInstant().toEpochMilli()

fun seedState(today: LocalDate = SEED_TODAY): PlannerState = SampleSeedData.build(today)

fun plannerCoreAt(nowMs: Long, today: LocalDate = SEED_TODAY): Pair<PlannerCore, FakeClock> {
    val clock = FakeClock(nowMs, today)
    return PlannerCore(clock) to clock
}

fun blockIdFor(taskId: String, date: LocalDate): String = "s_${taskId}_${yyyyMMdd(date)}"

/** Первый свободный слот на дате (без собственного блока). */
fun firstFreeSlot(
    state: PlannerState,
    date: LocalDate,
    durationMin: Int,
    excludeBlockId: String? = null,
): Pair<LocalTime, LocalTime> =
    ScheduleValidator.freeSlots(blocksOn(state, date), durationMin, excludeBlockId).first()

/** Лёгкий state для тестов производных функций. */
fun lightState(
    today: LocalDate = SEED_TODAY,
    tasks: List<Task> = emptyList(),
    blocks: List<ScheduledBlock> = emptyList(),
    checkIn: CheckIn? = null,
    dayPhase: DayPhase = DayPhase.RUNNING,
): PlannerState = PlannerState(
    todayDate = today,
    tasks = tasks,
    blocks = blocks,
    activeSession = null,
    factSessions = emptyList(),
    dayCheckIn = checkIn,
    dayPhase = dayPhase,
    dayMode = DayMode.NORMAL,
    dayEnergy = checkIn?.energy ?: 0,
    todayTop = emptyList(),
    quickNotesToday = emptyList(),
    endDayReview = null,
    planUi = PlanUiState(PlanMode.WEEK, today, YearMonth.from(today)),
    sheet = null,
    pendingDoElse = null,
)
