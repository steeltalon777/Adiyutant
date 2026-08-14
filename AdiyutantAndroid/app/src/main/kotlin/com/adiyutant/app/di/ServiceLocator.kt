package com.adiyutant.app.di

import com.adiyutant.app.core.CorePort
import com.adiyutant.app.core.fake.FakeCorePort
import com.adiyutant.app.data.InMemoryPlannerRepository
import com.adiyutant.app.data.PlannerRepository
import com.adiyutant.app.data.SampleSeedData
import com.adiyutant.app.domain.Clock
import com.adiyutant.app.domain.PlannerCore
import com.adiyutant.app.domain.SystemClock

/**
 * Manual dependency container (ADR-0002).
 *
 * Deliberately not a DI framework for the shell: one interface, one
 * implementation. Swap [CorePort] here when the real integration lands.
 */
object ServiceLocator {
    val corePort: CorePort by lazy { FakeCorePort() }

    val clock: Clock by lazy { SystemClock() }
    val plannerCore: PlannerCore by lazy { PlannerCore(clock) }
    val plannerRepository: PlannerRepository by lazy { InMemoryPlannerRepository(SampleSeedData.build(clock.today())) }
}
