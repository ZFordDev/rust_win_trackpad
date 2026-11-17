//! Just the native session. Window hooks and input delivery live elsewhere.

use std::{cell::Cell, marker::PhantomData, rc::Rc};

use windows::{
    core::{Error, Result},
    Win32::{
        Foundation::{HWND, RECT, RO_E_CLOSED},
        Graphics::DirectManipulation::*,
        System::Com::{
            CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
            COINIT_APARTMENTTHREADED,
        },
    },
};

/// Balance COM initialization on this thread, including a successful S_FALSE.
struct Apartment(PhantomData<Rc<()>>);

impl Drop for Apartment {
    fn drop(&mut self) {
        // SAFETY: Created only after successful initialization. The Rc marker
        // keeps this guard, and therefore the session, on the original thread.
        unsafe { CoUninitialize() };
    }
}

/// Own the interfaces until cleanup, then release them before the apartment.
pub(crate) struct Session {
    manager: IDirectManipulationManager,
    updates: IDirectManipulationUpdateManager,
    viewport: IDirectManipulationViewport,
    window: HWND,
    closed: Cell<bool>,
    _apartment: Apartment,
}

impl Session {
    /// Create and configure a session. Activation is a separate step.
    ///
    /// # Safety
    /// `window` must be live and owned by this thread. Call `close` before its
    /// destruction; the later window hook will handle that automatically.
    pub(crate) unsafe fn new(window: HWND) -> Result<Self> {
        // SAFETY: The caller supplies a live window on this thread. The guard
        // balances successful initialization, including failures further below.
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
            let apartment = Apartment(PhantomData);
            let manager: IDirectManipulationManager =
                CoCreateInstance(&DirectManipulationManager, None, CLSCTX_INPROC_SERVER)?;
            let updates = manager.GetUpdateManager()?;
            let viewport =
                manager.CreateViewport(None::<&IDirectManipulationFrameInfoProvider>, window)?;
            let session = Self {
                manager,
                updates,
                viewport,
                window,
                closed: Cell::new(false),
                _apartment: apartment,
            };
            // Own the session before configuration, so an error takes the same
            // cleanup path as a normal close.
            session.configure()?;
            Ok(session)
        }
    }

    fn configure(&self) -> Result<()> {
        // SAFETY: These interfaces belong to this thread's live session.
        unsafe {
            self.viewport.ActivateConfiguration(
                DIRECTMANIPULATION_CONFIGURATION_INTERACTION
                    | DIRECTMANIPULATION_CONFIGURATION_TRANSLATION_X
                    | DIRECTMANIPULATION_CONFIGURATION_TRANSLATION_Y
                    | DIRECTMANIPULATION_CONFIGURATION_TRANSLATION_INERTIA
                    | DIRECTMANIPULATION_CONFIGURATION_SCALING
                    | DIRECTMANIPULATION_CONFIGURATION_SCALING_INERTIA,
            )?;
            self.viewport
                .SetViewportOptions(DIRECTMANIPULATION_VIEWPORT_OPTIONS_MANUALUPDATE)?;
            // Apps own scroll bounds. This virtual surface is reset between
            // gestures by the input handler added in the next stage.
            self.viewport.SetViewportRect(&RECT {
                left: 0,
                top: 0,
                right: 1 << 20,
                bottom: 1 << 20,
            })?;
            let primary: IDirectManipulationPrimaryContent = self.viewport.GetPrimaryContent()?;
            primary.SetZoomBoundaries(0.1, 100.0)
        }
    }

    /// Activate the configured viewport. Input is hooked up separately.
    pub(crate) fn enable(&self) -> Result<()> {
        self.ensure_open()?;
        // SAFETY: The constructor's window contract still holds until close.
        unsafe {
            self.viewport.Enable()?;
            self.manager.Activate(self.window)
        }
    }

    /// Advance Windows' calculations. Scheduling belongs to the window timer.
    pub(crate) fn update(&self) -> Result<()> {
        self.ensure_open()?;
        // SAFETY: Called on the owning thread while the session is open.
        unsafe {
            self.updates
                .Update(None::<&IDirectManipulationFrameInfoProvider>)
        }
    }

    fn ensure_open(&self) -> Result<()> {
        if self.closed.get() {
            Err(Error::from_hresult(RO_E_CLOSED))
        } else {
            Ok(())
        }
    }

    /// Shut down once. Drop afterward must not touch the window again.
    pub(crate) fn close(&self) {
        if self.closed.replace(true) {
            return;
        }
        // SAFETY: The caller closes before window destruction. Setting the flag
        // first also makes reentrant or repeated cleanup harmless.
        unsafe {
            let _ = self.viewport.Stop();
            let _ = self.viewport.Abandon();
            let _ = self.manager.Deactivate(self.window);
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod tests;
