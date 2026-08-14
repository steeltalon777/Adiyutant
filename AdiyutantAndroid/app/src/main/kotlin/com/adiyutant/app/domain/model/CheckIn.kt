package com.adiyutant.app.domain.model

/**
 * Чек-ин (раздел 3.6 ТЗ). Первый чек-ин начинает день; последующие
 * обновляют текущее состояние. energy синхронизируется с KPI dayEnergy.
 */
data class CheckIn(
    val energy: Int,       // 0..3 (·, ⚡, ⚡⚡, ⚡⚡⚡)
    val focus: Int,        // 0..3 (туман, средний, острый, пик)
    val mood: Mood,        // BAD | OK | GOOD
    val obstacle: String?, // необязательно
    val atMs: Long,
)
