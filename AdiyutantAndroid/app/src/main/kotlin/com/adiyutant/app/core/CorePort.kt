package com.adiyutant.app.core

import com.adiyutant.app.core.model.CurrentActivity
import com.adiyutant.app.core.model.TodaySnapshot

/**
 * Boundary between the Android UI and the Adiyutant Core (ADR-0002).
 *
 * The UI consumes only this interface. The first implementation is
 * [com.adiyutant.app.core.fake.FakeCorePort] backed by sample data; a real
 * implementation backed by the Rust Core will replace it behind the same
 * contract (ADR-0003) without UI changes.
 *
 * All methods are suspend and return UI-facing models. No domain logic lives
 * here — the boundary mirrors Core facade use-cases, never re-implements them.
 */
interface CorePort {

    /** Snapshot of the current day for the "Сегодня" screen. */
    suspend fun getTodaySnapshot(): TodaySnapshot

    /** The activity the user is (or should be) doing right now. */
    suspend fun getCurrentActivity(): CurrentActivity
}
