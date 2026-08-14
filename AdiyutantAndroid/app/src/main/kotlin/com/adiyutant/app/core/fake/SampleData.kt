package com.adiyutant.app.core.fake

import java.time.LocalDate

/**
 * Static sample data used by [FakeCorePort].
 *
 * Values mirror the approved design mock (docs/design/today-screen.html)
 * so the shell can be visually checked against it without Rust.
 */
object SampleData {
    val today: LocalDate = LocalDate.of(2026, 6, 22)
}
