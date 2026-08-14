package com.adiyutant.app.domain

import java.time.LocalDate

/**
 * Абстракция времени для детерминированного тестирования (раздел 13.1 ТЗ).
 */
interface Clock {
    fun nowMs(): Long
    fun today(): LocalDate
}

/** Реальные часы. */
class SystemClock : Clock {
    override fun nowMs(): Long = System.currentTimeMillis()
    override fun today(): LocalDate = LocalDate.now()
}
