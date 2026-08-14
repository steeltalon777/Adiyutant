package com.adiyutant.app.di

import com.adiyutant.app.core.CorePort
import com.adiyutant.app.core.fake.FakeCorePort

/**
 * Manual dependency container (ADR-0002).
 *
 * Deliberately not a DI framework for the shell: one interface, one
 * implementation. Swap [CorePort] here when the real integration lands.
 */
object ServiceLocator {
    val corePort: CorePort by lazy { FakeCorePort() }
}
