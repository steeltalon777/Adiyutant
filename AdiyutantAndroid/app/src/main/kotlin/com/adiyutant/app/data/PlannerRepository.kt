package com.adiyutant.app.data

import com.adiyutant.app.domain.model.PlannerState

/**
 * Шов под будущий Rust Core (ADR-0004). In-memory реализация сменяется
 * Core-backed реализацией без изменения UI.
 */
interface PlannerRepository {
    fun load(): PlannerState
    fun save(state: PlannerState)
}
